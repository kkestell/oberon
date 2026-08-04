use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_RUN: AtomicUsize = AtomicUsize::new(0);

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
// The compiler runs from the repository root, which pins its runtime and
// build paths and keeps diagnostics stable. Each generated program runs in a
// fresh directory, so file operations cannot touch the checkout or another
// corpus case.
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

        let run = run(root, &source, &stem);
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

        let run = run(root, &source, &stem);
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

#[test]
fn deeply_nested_source_compiles_and_runs() {
    let parens = 850;
    generated_module(
        "DeepExprRegression",
        format!(
            "MODULE DeepExprRegression; VAR x: INTEGER; BEGIN x := {}1{} END DeepExprRegression.",
            "(".repeat(parens),
            ")".repeat(parens)
        ),
    );

    let ifs = 700;
    generated_module(
        "DeepIfRegression",
        format!(
            "MODULE DeepIfRegression; VAR x: INTEGER; BEGIN x := 0; {}x := 1{} END DeepIfRegression.",
            "IF x = 0 THEN ".repeat(ifs),
            " END".repeat(ifs)
        ),
    );
}

fn generated_module(name: &str, text: String) {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sequence = NEXT_RUN.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "oberon-generated-{}-{sequence}-{name}",
        std::process::id()
    ));
    fs::create_dir(&directory).unwrap_or_else(|e| panic!("creating {}: {e}", directory.display()));
    let source = directory.join(format!("{name}.Mod"));
    fs::write(&source, text).unwrap_or_else(|e| panic!("writing {}: {e}", source.display()));

    let compile = compile(root, &source);
    assert!(
        compile.status.success() && compile.stderr.is_empty(),
        "{name}: compile failed ({}):\n{}",
        compile.status,
        String::from_utf8_lossy(&compile.stderr)
    );

    let run = Command::new(root.join("build").join(name))
        .output()
        .unwrap_or_else(|e| panic!("running {name}: {e}"));
    assert!(
        run.status.success() && run.stdout.is_empty() && run.stderr.is_empty(),
        "{name}: run failed ({}), stdout {:?}, stderr {:?}",
        run.status,
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );

    fs::remove_dir_all(&directory)
        .unwrap_or_else(|e| panic!("removing {}: {e}", directory.display()));
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

fn run(root: &Path, source: &Path, stem: &str) -> std::process::Output {
    let sequence = NEXT_RUN.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "oberon-corpus-{}-{sequence}-{stem}",
        std::process::id()
    ));
    fs::create_dir(&directory)
        .unwrap_or_else(|e| panic!("creating isolated directory {}: {e}", directory.display()));
    assert!(
        directory.is_dir() && !directory.is_symlink(),
        "isolated run path is not the created directory: {}",
        directory.display()
    );

    let mut command = Command::new(root.join("build").join(stem));
    command.current_dir(&directory);
    let input = root.join(source).with_extension("stdin");
    if input.is_file() {
        command.stdin(Stdio::from(
            fs::File::open(&input).unwrap_or_else(|e| panic!("opening {}: {e}", input.display())),
        ));
    }
    let output = command
        .output()
        .unwrap_or_else(|e| panic!("running {stem} in {}: {e}", directory.display()));

    assert!(
        directory.is_dir() && !directory.is_symlink(),
        "isolated run path changed before cleanup: {}",
        directory.display()
    );
    fs::remove_dir_all(&directory)
        .unwrap_or_else(|e| panic!("removing isolated directory {}: {e}", directory.display()));
    output
}

fn expected(root: &Path, source: &Path) -> Vec<u8> {
    let path = root.join(source).with_extension("expected");
    fs::read(&path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}
