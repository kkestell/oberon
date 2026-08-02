mod ast;
mod diag;
mod driver;
mod ir;
mod lexer;
mod parser;
mod qbe;
mod sema;

use std::path::Path;

use anyhow::Result;
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: oberon <file.Mod>");
        std::process::exit(2);
    };
    driver::build(Path::new(&path))
}
