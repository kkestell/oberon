use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// Two corpora, one test. Failures are collected so one broken module doesn't
// hide the rest.
//
//   tests/corpus/X.Mod  must compile and run; X.expected is the program's stdout.
//   tests/errors/X.Mod  must fail to compile; X.expected is the compiler's stderr.
//
// Everything runs from the repo root, which pins the driver's relative
// runtime/oberon.c and build/ paths and keeps the source paths the compiler
// prints in diagnostics stable across machines.
#[test]
fn corpus() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut failures = Vec::new();

    for source in modules(root, "tests/corpus") {
        let stem = stem(&source);
        let compile = compile(root, &source);
        if !compile.status.success() {
            failures.push(format!(
                "{stem}: compile failed ({}):\n{}",
                compile.status,
                String::from_utf8_lossy(&compile.stderr)
            ));
            continue;
        }

        let run = Command::new(root.join("build").join(&stem))
            .output()
            .expect("running compiled module");
        let expected = expected(root, &source);
        if run.stdout != expected {
            failures.push(format!(
                "{stem}: expected stdout {:?}, got {:?} (exit {})",
                String::from_utf8_lossy(&expected),
                String::from_utf8_lossy(&run.stdout),
                run.status
            ));
        }
    }

    for source in modules(root, "tests/errors") {
        let stem = stem(&source);
        let compile = compile(root, &source);
        if compile.status.success() {
            failures.push(format!(
                "{stem}: expected compilation to fail, but it succeeded"
            ));
            continue;
        }
        let expected = expected(root, &source);
        if compile.stderr != expected {
            failures.push(format!(
                "{stem}: expected stderr {:?}, got {:?}",
                String::from_utf8_lossy(&expected),
                String::from_utf8_lossy(&compile.stderr)
            ));
        }
    }

    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

fn modules(root: &Path, dir: &str) -> Vec<PathBuf> {
    // Relative to the root, so the compiler prints "tests/errors/X.Mod:2:8: ..."
    // and the .expected files are not tied to one checkout location.
    let mut sources: Vec<_> = fs::read_dir(root.join(dir))
        .unwrap_or_else(|e| panic!("reading {dir}: {e}"))
        .map(|e| Path::new(dir).join(e.expect("reading dir entry").file_name()))
        .filter(|p| p.extension().is_some_and(|e| e == "Mod"))
        .collect();
    sources.sort();
    assert!(!sources.is_empty(), "no .Mod files in {dir}");
    sources
}

fn stem(source: &Path) -> String {
    source.file_stem().unwrap().to_str().unwrap().to_string()
}

fn compile(root: &Path, source: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_oberon"))
        .arg(source)
        .current_dir(root)
        .output()
        .expect("running compiler")
}

fn expected(root: &Path, source: &Path) -> Vec<u8> {
    let path = root.join(source).with_extension("expected");
    fs::read(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}
