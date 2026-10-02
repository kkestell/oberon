# Program arguments and exit status

## Plan

`agents/plans/2026-10-02-020-program-arguments-and-exit-status.md`

## Summary

Added the bundled `Program` module with `count`, `Arg`, and `Exit`, backed by three private runtime operations. The generated `main` now receives `argc` and `argv` and hands them to `oberon_init`. The plan's goal is met.

## Checks run

- `make test` — 43 unit tests and 5 integration tests pass, including `program_arguments_and_exit_status` and the two new `tests/failures` cases.
- `cargo fmt --check` — Clean after `cargo fmt`.
- `cargo clippy --all-targets -- -D warnings` — Clean.

## Manual verification

1. A program's output survives `Exit` when stdout is redirected to a file, and the status reaches the shell. `make test` does not rebuild the release compiler, so run `make` first.

   ```sh
   make
   cd "$(mktemp -d)"
   printf 'MODULE Echo; IMPORT Out, Program; VAR i, res: INTEGER; a: ARRAY 64 OF CHAR;\nBEGIN FOR i := 0 TO Program.count - 1 DO Program.Arg(i, a, res); Out.String(a); Out.Ln END; Program.Exit(Program.count) END Echo.\n' > Echo.Mod
   ~/projects/oberon/target/release/oberon Echo.Mod && ./Echo x "y z" > out.txt; echo "status $?"; cat out.txt
   ```

   Printed `status 2`, then `x` and `y z`.
