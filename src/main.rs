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

    let Some(path) = std::env::args().nth(1).map(PathBuf::from) else {
        eprintln!("usage: oberon <file.Mod>");
        std::process::exit(2);
    };
    // Parsing and semantic analysis follow the recursive shape of the source.
    // A modestly larger stack keeps deeply nested, valid Oberon source from
    // inheriting the host main thread's comparatively small stack limit.
    match std::thread::Builder::new()
        .name("compiler".into())
        .stack_size(32 * 1024 * 1024)
        .spawn(move || driver::build(&path))
        .context("starting compiler thread")?
        .join()
    {
        Ok(result) => result,
        Err(panic) => std::panic::resume_unwind(panic),
    }
}
