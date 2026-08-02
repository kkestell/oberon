use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

const QBE: &str = "qbe";
const CC: &str = "cc";
const RUNTIME_C: &str = "runtime/oberon.c";
const BUILD_DIR: &str = "build";

// Hardcoded until the lexer and parser exist. Calls oberon_init, allocates
// through the GC, round-trips 42 through the heap, and returns it as the exit
// status — so a live allocation is on the path to the observable result.
const IL: &str = r#"export function w $main() {
@start
        call $oberon_init()
        %p =l call $oberon_alloc(l 8)
        storew 42, %p
        %v =w loadw %p
        ret %v
}
"#;

pub fn build() -> Result<()> {
    let build_dir = Path::new(BUILD_DIR);
    fs::create_dir_all(build_dir).context("creating build directory")?;

    let il = build_dir.join("out.ssa");
    let asm = build_dir.join("out.s");
    let exe = build_dir.join("out");

    // TODO: --emit-il should write this to stdout instead.
    fs::write(&il, IL).context("writing QBE IL")?;

    run(Command::new(QBE).arg("-o").arg(&asm).arg(&il), QBE)?;
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

fn run(cmd: &mut Command, name: &str) -> Result<()> {
    tracing::debug!("{:?}", cmd);
    let status = cmd
        .status()
        .with_context(|| format!("running {name}"))?;
    if !status.success() {
        bail!("{name} failed: {status}");
    }
    Ok(())
}
