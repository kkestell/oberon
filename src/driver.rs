use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::diag::Diagnostic;
use crate::{lexer, parser, qbe, sema};

const QBE: &str = "qbe";
const CC: &str = "cc";
const RUNTIME_C: &str = "runtime/oberon.c";
const BUILD_DIR: &str = "build";

pub fn build(source: &Path) -> Result<()> {
    let src =
        fs::read_to_string(source).with_context(|| format!("reading {}", source.display()))?;

    let mut diags = Vec::new();
    let toks = lexer::lex(&src, &mut diags);
    // Stop here rather than parsing on. The lexer drops a token it could not
    // read, so the stream has a hole in it and the parser would report a
    // second, invented error next to the real one.
    report(source, &diags)?;

    let module = match parser::parse(toks) {
        Ok(m) => m,
        Err(d) => return report(source, &[d]), // always Err: the list is non-empty
    };
    tracing::debug!("{module:#?}");

    let (scope, diags) = sema::analyze(&module);
    report(source, &diags)?;

    let il = qbe::emit(&module, scope);

    let build_dir = Path::new(BUILD_DIR);
    fs::create_dir_all(build_dir).context("creating build directory")?;
    let ssa = build_dir.join(format!("{}.ssa", module.name));
    let asm = build_dir.join(format!("{}.s", module.name));
    let exe = build_dir.join(&module.name);

    // TODO: --emit-il should write this to stdout instead.
    fs::write(&ssa, &il).context("writing QBE IL")?;

    run(Command::new(QBE).arg("-o").arg(&asm).arg(&ssa), QBE)?;
    run(
        Command::new(CC)
            .arg(&asm)
            .arg(RUNTIME_C)
            .arg("-lgc")
            .arg("-o")
            .arg(&exe),
        CC,
    )?;

    Ok(())
}

fn report(source: &Path, diags: &[Diagnostic]) -> Result<()> {
    if diags.is_empty() {
        return Ok(());
    }
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
