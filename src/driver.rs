use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::diag::Diagnostic;
use crate::{ast, ir, lexer, parser, qbe, sema};

const QBE: &str = "qbe";
const CC: &str = "cc";
// Relative to the directory above the compiler's own executable, so an
// install under a prefix and a Cargo build under target/ resolve alike.
const SUPPORT_DIR: &str = "lib/oberon";
const RUNTIME_ARCHIVE: &str = "liboberon.a";
// The one module with no Oberon source. Its procedures are the runtime's.
const NATIVE_OUT: &str = "Out";

pub fn build(source: &Path, output: &Path) -> Result<()> {
    let name = source
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();
    let support = support_dir()?;
    let mut build = Build {
        done: HashMap::new(),
        modules: Vec::new(),
    };
    build.compile(&name, source)?;

    let program = ir::Program {
        modules: build.modules,
    };
    tracing::debug!("{program:#?}");
    let il = qbe::emit(&program);

    // The intermediates are not a product of the build, so they live only as
    // long as it does and never land beside the source or the output. The
    // name is predictable and /tmp is shared on Linux, so an existing path is
    // refused: another user's directory or symlink could swap the assembly.
    let temp = std::env::temp_dir().join(format!("oberon-{}", std::process::id()));
    fs::create_dir(&temp)
        .with_context(|| format!("creating temporary directory {}", temp.display()))?;
    let result = assemble(&support, &temp, &name, &il, output);
    let removed = fs::remove_dir_all(&temp).context("removing temporary directory");
    result.and(removed)
}

fn assemble(support: &Path, temp: &Path, name: &str, il: &str, output: &Path) -> Result<()> {
    let ssa = temp.join(format!("{name}.ssa"));
    let asm = temp.join(format!("{name}.s"));

    // TODO: --emit-il should write this to stdout instead.
    fs::write(&ssa, il).context("writing QBE IL")?;

    run(
        Command::new(support.join(QBE))
            .arg("-o")
            .arg(&asm)
            .arg(&ssa),
        QBE,
    )?;
    run(
        Command::new(CC)
            .arg(&asm)
            .arg(support.join(RUNTIME_ARCHIVE))
            .arg("-o")
            .arg(output),
        CC,
    )
}

// The executable is canonicalized first, so a symlink to an installed
// compiler still finds the support directory of that install.
fn support_dir() -> Result<PathBuf> {
    let exe = std::env::current_exe()
        .and_then(fs::canonicalize)
        .context("locating the compiler executable")?;
    let dir = exe
        .parent()
        .and_then(Path::parent)
        .expect("executable path has a parent directory")
        .join(SUPPORT_DIR);
    if !dir.is_dir() {
        bail!("cannot find the support directory {}", dir.display());
    }
    Ok(dir)
}

// One invocation's module graph. Nothing here survives the process: the next
// build rereads and reanalyzes every source it needs, and no interface is
// written to disk.
struct Build {
    done: HashMap<String, sema::Interface>,
    // Source modules in the order their initializers must run. A dependency
    // finishes before its client starts, so pushing on completion puts them
    // in dependency-first order.
    modules: Vec<ir::Module>,
}

impl Build {
    // Compiles one source module and everything it imports. Diagnostics are
    // reported against the file that owns them and end the build, so a client
    // is never analyzed against a half-analyzed dependency.
    fn compile(&mut self, name: &str, path: &Path) -> Result<()> {
        let src =
            fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;

        let mut diags = Vec::new();
        let toks = lexer::lex(&src, &mut diags);
        // Stop here rather than parsing on. The lexer drops a token it could
        // not read, so the stream has a hole in it and the parser would report
        // a second, invented error next to the real one.
        report(path, &diags)?;

        let module = match parser::parse(toks) {
            Ok(m) => m,
            Err(d) => return fail(path, &[d]),
        };
        tracing::debug!("{module:#?}");

        // Source lookup goes by file name, so every module must have the exact
        // file name that another module would use to import it.
        let expected_filename = format!("{}.Mod", module.name);
        if module.name != name
            || path.file_name().and_then(|name| name.to_str()) != Some(expected_filename.as_str())
        {
            return fail(
                path,
                &[Diagnostic::new(
                    module.pos,
                    format!(
                        "module '{}' must be stored in a file named '{}.Mod'",
                        module.name, module.name
                    ),
                )],
            );
        }

        let resolved = self.dependencies(&module, path)?;
        let (lowered, interface) = match sema::analyze(&module, &resolved) {
            Ok(result) => result,
            Err(diags) => return fail(path, &diags),
        };

        self.modules.push(lowered);
        self.done.insert(name.to_string(), interface);
        Ok(())
    }

    fn dependencies(
        &mut self,
        module: &ast::Module,
        path: &Path,
    ) -> Result<HashMap<String, sema::Interface>> {
        let mut resolved = HashMap::new();
        for import in &module.imports {
            if !self.done.contains_key(&import.name) {
                if import.name == NATIVE_OUT {
                    self.done.insert(import.name.clone(), sema::out_interface());
                } else {
                    return fail(
                        path,
                        &[Diagnostic::new(
                            import.pos,
                            format!("cannot find module '{}'", import.name),
                        )],
                    );
                }
            }
            resolved.insert(import.name.clone(), self.done[&import.name].clone());
        }
        Ok(resolved)
    }
}

fn report(source: &Path, diags: &[Diagnostic]) -> Result<()> {
    if diags.is_empty() {
        return Ok(());
    }
    fail(source, diags)
}

// Never returns Ok. The type parameter only lets a caller in the middle of
// producing something else end the build with `return fail(...)`.
fn fail<T>(source: &Path, diags: &[Diagnostic]) -> Result<T> {
    for d in diags {
        eprintln!(
            "{}:{}:{}: {}",
            source.display(),
            d.pos.line,
            d.pos.col,
            d.msg
        );
    }
    bail!("{} error(s)", diags.len());
}

fn run(cmd: &mut Command, name: &str) -> Result<()> {
    tracing::debug!("{:?}", cmd);
    let status = cmd.status().with_context(|| format!("running {name}"))?;
    if !status.success() {
        bail!("{name} failed: {status}");
    }
    Ok(())
}
