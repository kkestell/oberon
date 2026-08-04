use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::diag::Diagnostic;
use crate::{ast, ir, lexer, parser, qbe, sema};

const QBE: &str = "qbe";
const CC: &str = "cc";
const RUNTIME_C: &str = "runtime/oberon.c";
const STANDARD_C: &str = "runtime/standard.c";
const BUILD_DIR: &str = "build";
const LIB_DIR: &str = "lib";
const PRIVATE_RUNTIME: &str = "OberonRuntime";

pub fn build(source: &Path) -> Result<()> {
    let name = source
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default()
        .to_string();
    let mut build = Build {
        root_dir: source.parent().unwrap_or(Path::new("")).to_path_buf(),
        lib_dir: PathBuf::from(LIB_DIR),
        active: Vec::new(),
        done: HashMap::new(),
        modules: Vec::new(),
    };
    let root_source = std::path::absolute(source).context("resolving root source path")?;
    let bundled_source = std::path::absolute(Path::new(LIB_DIR).join(format!("{name}.Mod")))
        .context("resolving bundled source path")?;
    let origin = if root_source == bundled_source {
        Origin::Bundled
    } else {
        Origin::User
    };
    build.compile(&name, source, origin)?;

    let program = ir::Program {
        modules: build.modules,
    };
    tracing::debug!("{program:#?}");
    // Each module's own globals already fit the target limit, but a build is
    // one data image and several modules can cross it together. Catching that
    // here keeps it a compiler error rather than a linker relocation failure.
    check_static_data(&program)?;
    let il = qbe::emit(&program);

    let build_dir = Path::new(BUILD_DIR);
    fs::create_dir_all(build_dir).context("creating build directory")?;
    let ssa = build_dir.join(format!("{name}.ssa"));
    let asm = build_dir.join(format!("{name}.s"));
    let exe = build_dir.join(&name);

    // TODO: --emit-il should write this to stdout instead.
    fs::write(&ssa, &il).context("writing QBE IL")?;

    run(Command::new(QBE).arg("-o").arg(&asm).arg(&ssa), QBE)?;
    run(
        Command::new(CC)
            .arg(&asm)
            .arg(RUNTIME_C)
            .arg(STANDARD_C)
            .arg("-lgc")
            // The REAL operations of Report 10.2 are the C float forms.
            .arg("-lm")
            .arg("-o")
            .arg(&exe),
        CC,
    )?;

    Ok(())
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Origin {
    User,
    Bundled,
}

struct Found {
    path: PathBuf,
    origin: Origin,
}

// One invocation's module graph. Nothing here survives the process: the next
// build rereads and reanalyzes every source it needs, and no interface is
// written to disk.
struct Build {
    root_dir: PathBuf,
    lib_dir: PathBuf,
    // The modules whose compilation has started and not finished, innermost
    // last. An import naming one of these closes a cycle.
    active: Vec<String>,
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
    fn compile(&mut self, name: &str, path: &Path, origin: Origin) -> Result<()> {
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

        self.active.push(name.to_string());
        let resolved = self.dependencies(&module, path, origin)?;
        let (lowered, interface) = match sema::analyze(&module, &resolved) {
            Ok(result) => result,
            Err(diags) => return fail(path, &diags),
        };
        self.active.pop();

        self.modules.push(lowered);
        self.done.insert(name.to_string(), interface);
        Ok(())
    }

    // Depth first, in written order, so sibling dependencies initialize
    // left to right. A module reached twice is compiled once.
    fn dependencies(
        &mut self,
        module: &ast::Module,
        path: &Path,
        origin: Origin,
    ) -> Result<HashMap<String, sema::Interface>> {
        let mut resolved = HashMap::new();
        for import in &module.imports {
            // The private native interface belongs only to actual bundled
            // source. It is returned directly and never enters `done`, so it
            // cannot be captured by or leak into an ordinary module graph.
            if origin == Origin::Bundled && import.name == PRIVATE_RUNTIME {
                resolved.insert(import.name.clone(), sema::runtime_interface());
                continue;
            }
            if !self.done.contains_key(&import.name) {
                if let Some(cycle) = self.cycle(&import.name) {
                    return fail(
                        path,
                        &[Diagnostic::new(
                            import.pos,
                            format!("import cycle: {cycle}"),
                        )],
                    );
                }
                match lookup(&self.root_dir, &self.lib_dir, &import.name) {
                    Some(found) => self.compile(&import.name, &found.path, found.origin)?,
                    None => {
                        return fail(
                            path,
                            &[Diagnostic::new(
                                import.pos,
                                format!("cannot find module '{}'", import.name),
                            )],
                        );
                    }
                }
            }
            resolved.insert(import.name.clone(), self.done[&import.name].clone());
        }
        Ok(resolved)
    }

    // The cycle text starts where the repeated name first appears, so an
    // acyclic prefix leading into the cycle is left out. A direct self-import
    // is the two-name form of the same text.
    fn cycle(&self, name: &str) -> Option<String> {
        let start = self.active.iter().position(|active| active == name)?;
        let mut chain: Vec<&str> = self.active[start..].iter().map(String::as_str).collect();
        chain.push(name);
        Some(chain.join(" -> "))
    }
}

// The whole program's static data, each global padded to its own alignment.
// The running total stops at the first global that crosses the limit, and
// every global is itself within it, so the sum cannot run away. This error has
// no source position: it belongs to the build, not to any one declaration.
fn check_static_data(program: &ir::Program) -> Result<()> {
    let mut total: i64 = 0;
    for module in &program.modules {
        if !module.descriptors.is_empty() {
            total = (total + 7) / 8 * 8
                + i64::try_from(module.descriptors.len()).expect("descriptor count fits") * 8;
        }
        if total > ir::MAX_OBJECT_SIZE {
            bail!("program static data exceeds target object-size limit");
        }
        for global in &module.globals {
            let align = global.ty.align();
            total = (total + align - 1) / align * align + global.ty.size();
            if total > ir::MAX_OBJECT_SIZE {
                bail!("program static data exceeds target object-size limit");
            }
        }
        // String literals are data too. Each is byte-aligned and one byte
        // longer than its characters, for the terminator.
        for literal in &module.literals {
            total += literal.bytes.len() as i64 + 1;
            if total > ir::MAX_OBJECT_SIZE {
                bail!("program static data exceeds target object-size limit");
            }
        }
    }
    Ok(())
}

// Report 11 leaves module lookup to the implementation. Two fixed directories,
// root first so an application can supply a module that shadows a bundled one.
// Spelling is exact, and neither the importing file's own directory nor any
// environment variable takes part.
fn lookup(root_dir: &Path, lib_dir: &Path, name: &str) -> Option<Found> {
    let file = format!("{name}.Mod");
    let root = root_dir.join(&file);
    if root.is_file() {
        let bundled = lib_dir.join(&file);
        return Some(Found {
            path: root,
            origin: if same_path(&root_dir.join(&file), &bundled) {
                Origin::Bundled
            } else {
                Origin::User
            },
        });
    }
    let bundled = lib_dir.join(file);
    bundled.is_file().then_some(Found {
        path: bundled,
        origin: Origin::Bundled,
    })
}

fn same_path(a: &Path, b: &Path) -> bool {
    match (std::path::absolute(a), std::path::absolute(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
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
//
// Diagnostics arrive in the order the analyzer produced them, which is not
// always source order: a pointer's forward base is diagnosed after the whole
// TYPE section that named it, so its message would otherwise print after
// messages from later lines. Sorting by position means the list always reads
// down the file. The sort is stable, so two diagnostics at one position keep
// the order they were reported in.
fn fail<T>(source: &Path, diags: &[Diagnostic]) -> Result<T> {
    let mut ordered: Vec<&Diagnostic> = diags.iter().collect();
    ordered.sort_by_key(|d| (d.pos.line, d.pos.col));
    for d in ordered {
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

#[cfg(test)]
mod tests {
    use super::*;

    // tests/lookup/root and tests/lookup/lib hold Both.Mod, and only the
    // library holds LibOnly.Mod.
    fn dirs() -> (PathBuf, PathBuf) {
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/lookup");
        (base.join("root"), base.join("lib"))
    }

    #[test]
    fn root_directory_wins() {
        let (root, lib) = dirs();
        let found = lookup(&root, &lib, "Both").expect("module exists");
        assert_eq!(found.path, root.join("Both.Mod"));
        assert_eq!(found.origin, Origin::User);
    }

    #[test]
    fn library_is_the_fallback() {
        let (root, lib) = dirs();
        let found = lookup(&root, &lib, "LibOnly").expect("module exists");
        assert_eq!(found.path, lib.join("LibOnly.Mod"));
        assert_eq!(found.origin, Origin::Bundled);
    }

    #[test]
    fn configured_library_file_keeps_bundled_origin_when_it_is_also_root() {
        let (_, lib) = dirs();
        let found = lookup(&lib, &lib, "LibOnly").expect("module exists");
        assert_eq!(found.origin, Origin::Bundled);
    }

    #[test]
    fn spelling_is_exact() {
        let (root, lib) = dirs();
        assert!(lookup(&root, &lib, "both").is_none());
    }

    #[test]
    fn missing_module_is_not_found() {
        let (root, lib) = dirs();
        assert!(lookup(&root, &lib, "Absent").is_none());
    }
}
