use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

// Three corpora, one test. Failures are collected so one broken module doesn't
// hide the rest.
//
//   tests/corpus/X.Mod  must compile and run; X.expected is the program's stdout.
//   tests/errors/X.Mod  must fail to compile; X.expected is the compiler's stderr.
//   tests/failures/X.Mod must compile and fail at runtime; X.expected is stderr.
//
// A module graph lives in its own subdirectory. Only a .Mod file with a
// sibling .expected file is a test root; the rest of the directory is the
// dependencies that root imports.
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
        if !compile.status.success() || !compile.stderr.is_empty() {
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
        if !run.status.success() || !run.stderr.is_empty() || run.stdout != expected {
            failures.push(format!(
                "{stem}: expected stdout {:?}, got {:?} (exit {}, stderr {:?})",
                String::from_utf8_lossy(&expected),
                String::from_utf8_lossy(&run.stdout),
                run.status,
                String::from_utf8_lossy(&run.stderr)
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

    for source in modules(root, "tests/failures") {
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
        if run.status.success() {
            failures.push(format!(
                "{stem}: expected runtime failure, but it exited successfully"
            ));
        }
        let expected = expected(root, &source);
        if run.stderr != expected {
            failures.push(format!(
                "{stem}: expected stderr {:?}, got {:?}",
                String::from_utf8_lossy(&expected),
                String::from_utf8_lossy(&run.stderr)
            ));
        }
    }

    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

#[test]
fn root_filename_must_end_in_mod() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = Path::new("tests/errors/RootWrongExtension.txt");
    let compile = compile(root, source);

    assert!(
        !compile.status.success(),
        "{} unexpectedly compiled successfully",
        source.display()
    );
    assert_eq!(compile.stderr, expected(root, source));
}

fn modules(root: &Path, dir: &str) -> Vec<PathBuf> {
    // Relative to the root, so the compiler prints "tests/errors/X.Mod:2:8: ..."
    // and the .expected files are not tied to one checkout location.
    let mut sources = Vec::new();
    collect(root, Path::new(dir), &mut sources);
    sources.sort();
    assert!(!sources.is_empty(), "no test roots in {dir}");
    sources
}

fn collect(root: &Path, dir: &Path, sources: &mut Vec<PathBuf>) {
    let entries =
        fs::read_dir(root.join(dir)).unwrap_or_else(|e| panic!("reading {}: {e}", dir.display()));
    for entry in entries {
        let name = entry.expect("reading dir entry").file_name();
        let relative = dir.join(&name);
        if root.join(&relative).is_dir() {
            collect(root, &relative, sources);
        } else if relative.extension().is_some_and(|e| e == "Mod")
            && root.join(&relative).with_extension("expected").is_file()
        {
            sources.push(relative);
        }
    }
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
