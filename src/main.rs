mod ast;
mod diag;
mod driver;
mod ir;
mod lexer;
mod parser;
mod qbe;
mod sema;

use std::path::PathBuf;

use anyhow::{Context, Result};
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let Some((path, output)) = arguments() else {
        eprintln!("usage: oberon [-o output] <file.Mod>");
        std::process::exit(2);
    };
    // Parsing and semantic analysis follow the recursive shape of the source.
    // A modestly larger stack keeps deeply nested, valid Oberon source from
    // inheriting the host main thread's comparatively small stack limit.
    match std::thread::Builder::new()
        .name("compiler".into())
        .stack_size(32 * 1024 * 1024)
        .spawn(move || driver::build(&path, &output))
        .context("starting compiler thread")?
        .join()
    {
        Ok(result) => result,
        Err(panic) => std::panic::resume_unwind(panic),
    }
}

// Without -o, the executable lands in the working directory, named after the
// source file.
fn arguments() -> Option<(PathBuf, PathBuf)> {
    let mut source = None;
    let mut output = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "-o" && output.is_none() {
            output = Some(PathBuf::from(args.next()?));
        } else if !arg.starts_with('-') && source.is_none() {
            source = Some(PathBuf::from(arg));
        } else {
            return None;
        }
    }
    let source: PathBuf = source?;
    let output = output.unwrap_or_else(|| PathBuf::from(source.file_stem().unwrap_or_default()));
    Some((source, output))
}
