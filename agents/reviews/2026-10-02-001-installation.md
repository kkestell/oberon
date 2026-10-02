# Installation review

## Scope and coverage

Commit `db6d32e`, "Install the compiler with vendored QBE and BDWGC": `Makefile`, `src/driver.rs`, `src/main.rs`, `tests/corpus.rs`, `.gitignore`, `vendor/README.md`, and the `AGENTS.md` and roadmap updates, read against `agents/plans/2026-10-02-019-installation.md`. Also checked the documentation that tells contributors how to build and run the compiler (`bugs/prompt.md`, `bugs/prompt-volume.md`).

The vendored `vendor/bdwgc/` and `vendor/qbe/` trees were not compared against their release tarballs. Linux was tested on aarch64 Debian only, not x86-64.

## Fixed

- **Cleanup failure hides the build failure** (`src/driver.rs:59`): if QBE or `cc` failed and removing the temporary directory then also failed, `build` returned only the removal error, so the real build error was lost. `build` now returns the build error first and reports the cleanup error only when the build succeeded.
- **Predictable shared temporary directory** (`src/driver.rs:55`): intermediates went in `temp_dir()/oberon-<pid>`, created with `create_dir_all`, which accepts an existing directory or a symlink to one. On Linux `temp_dir()` is the shared `/tmp`, so another local user could create that path ahead of time as a symlink to a directory they control and swap the `.s` between QBE and `cc`, injecting code into the output executable. Two builds in separate PID namespaces sharing `/tmp` could also delete each other's intermediates. A standalone check confirmed `create_dir_all` returns `Ok` on a pre-placed symlink and writes land in its target. The directory is now created with `fs::create_dir`, which refuses an existing path; the worst a squatter can do is fail one build, with an error naming the path.
- **Bug-hunt prompts described the old build** (`bugs/prompt.md:68`, `bugs/prompt-volume.md:43`): both prompts said the compiler writes `build/<Module>` and must run from the repository root, told the reader to export Homebrew `CPATH`/`LIBRARY_PATH`, and built with `cargo build`. The compiler now writes `./<Module>` unless `-o` is given and needs `make` to stage `target/lib/oberon/`, so the batch runner would have scattered executables in the repository root and reported every program as `RUN ... status=127`. The prompts now build with `make test`, create `build/`, pass `-o build/<Module>` in every compile command, and drop the Homebrew export.

## Findings

No open findings.

## Checks run

- `make test` on macOS ARM64, before and after the fix: 43 unit and 4 corpus tests pass.
- `make test` from a clean `git archive` of `db6d32e` in a `rust:1-bookworm` (aarch64 Linux) container, with crates vendored from the host: QBE, the collector, and `liboberon.a` build; 43 unit and 4 corpus tests pass, including `compiles_outside_the_checkout`.
- `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`: clean.
- A standalone Rust check of `create_dir_all` and `create_dir` against a pre-placed symlink.
- After the temporary-directory fix: a compile run through `exec` under a PID whose `oberon-<pid>` directory already existed failed with "creating temporary directory ...: File exists"; an ordinary compile and run still work, and `make test` passes.
- The updated `bugs/prompt.md` baseline loop over `tests/corpus`: every case compiles into `build/`.

## Verdict

The installation works as planned on macOS and on aarch64 Linux. All three findings were fixed; nothing is open.
