# Installation review

## Scope and coverage

Commit `db6d32e`, "Install the compiler with vendored QBE and BDWGC": `Makefile`, `src/driver.rs`, `src/main.rs`, `tests/corpus.rs`, `.gitignore`, `vendor/README.md`, and the `AGENTS.md` and roadmap updates, read against `agents/plans/2026-10-02-019-installation.md`. Also checked the documentation that tells contributors how to build and run the compiler (`bugs/prompt.md`, `bugs/prompt-volume.md`).

The vendored `vendor/bdwgc/` and `vendor/qbe/` trees were not compared against their release tarballs. Linux was tested on aarch64 Debian only, not x86-64.

## Fixed

- **Cleanup failure hides the build failure** (`src/driver.rs:56`): if QBE or `cc` failed and removing the temporary directory then also failed, `build` returned only the removal error, so the real build error was lost. `build` now returns the build error first and reports the cleanup error only when the build succeeded.

## Findings

### medium

#### security

- **Predictable shared temporary directory** (`src/driver.rs:52`): intermediates go in `temp_dir()/oberon-<pid>`, created with `create_dir_all`, which accepts an existing directory or a symlink to one. On Linux `temp_dir()` is the shared `/tmp`. Another local user can create `/tmp/oberon-<pid>` ahead of time as a symlink to a directory they control, and the build writes its `.ssa` and `.s` there. Because QBE writes the `.s` before `cc` reads it, that user can swap the assembly and inject code into the output executable. The same reuse lets two builds in separate PID namespaces that share `/tmp` collide, and one deletes the other's intermediates mid-build. A standalone check confirmed that `create_dir_all` returns `Ok` on a pre-placed symlink and that writes then land in its target, while `create_dir` returns `AlreadyExists`. Suggested fix: create the directory with `fs::create_dir` under a name that is not guessable, or use the `tempfile` crate, so an existing path is never reused.

#### documentation

- **Bug-hunt prompts describe the old build** (`bugs/prompt.md:104`, `bugs/prompt-volume.md:152`): both prompts say the compiler writes `build/<Module>` and has to run from the repository root, and they tell the reader to export Homebrew `CPATH`/`LIBRARY_PATH` and build with `cargo build`. The compiler now writes `./<Module>` to the working directory unless `-o` is given. An agent following `prompt-volume.md`'s batch runner would scatter executables in the repository root and report every program as `RUN ... status=127` from the missing `./build/$m`. Neither prompt mentions `make`, so on a fresh checkout `target/lib/oberon` is missing and every compile fails with "cannot find the support directory". Suggested fix: run `make` (or `make test`) in place of `cargo build`/`cargo test`, pass `-o "build/$m"` in the runners and examples, drop the Homebrew export, and rewrite the "working directory must be the repository root" note.

## Checks run

- `make test` on macOS ARM64, before and after the fix: 43 unit and 4 corpus tests pass.
- `make test` from a clean `git archive` of `db6d32e` in a `rust:1-bookworm` (aarch64 Linux) container, with crates vendored from the host: QBE, the collector, and `liboberon.a` build; 43 unit and 4 corpus tests pass, including `compiles_outside_the_checkout`.
- `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`: clean.
- A standalone Rust check of `create_dir_all` and `create_dir` against a pre-placed symlink.

## Verdict

The installation works as planned on macOS and on aarch64 Linux. One error-handling slip was fixed. The temporary-directory handling should stop reusing a predictable path before the compiler runs on shared Linux hosts, and the bug-hunt prompts need updating to the new build.
