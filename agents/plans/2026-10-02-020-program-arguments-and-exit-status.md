# Program arguments and exit status

## Goal

Programs cannot read their command-line arguments or choose an exit status: the generated `main` takes no parameters and always returns 0. After this change, a bundled `Program` module exposes the argument count, copies an argument into a character array, and ends the program with a chosen status.

## Related code

- `src/qbe.rs` — `emit_program` writes `export function w $main()`, calls `$oberon_init()`, runs every module body, and returns 0.
- `runtime/oberon.c` — `oberon_init` configures and starts the collector.
- `runtime/standard.c` — the private library interface; `oberon_lib_file_error` is the model for copying a bounded string into an open array.
- `src/sema/symbols.rs` — `runtime_interface` declares the typed `OberonRuntime` members, and `native_symbol` maps `ProgramArg` to `oberon_lib_program_arg`.
- `lib/In.Mod` — an exported variable that clients read but cannot assign; `lib/Strings.Mod` — preconditions written as `ASSERT`.
- `tests/corpus.rs` — the corpus harness has no way to pass arguments or check an exit status; `compiles_outside_the_checkout` is the model for a dedicated test.
- `references/compilers/obnc/lib/obnc/ext/extArgs.obn` and `extArgs.c` — OBNC's `count` and `Get`, which this interface follows.

## Decisions

**Interface.**

```oberon
MODULE Program;
VAR count*: INTEGER;
PROCEDURE Arg*(n: INTEGER; VAR arg: ARRAY OF CHAR; VAR res: INTEGER);
PROCEDURE Exit*(status: INTEGER);
```

`count` is the number of arguments after the program name, set by the module body, so argument 0 is the first user argument and the program name is not available. Clients read `count` but cannot assign it (Report 11).

**`Arg` truncation.** `Arg` requires `0 <= n < count`; a violation is `assertion failed`. A nonempty destination receives a terminated prefix of at most `LEN(arg) - 1` bytes. A zero-length destination is untouched. `res` is the number of argument bytes not copied, so zero means the whole argument arrived. Unlike OBNC, which writes `arg[-1]` for a zero-length destination, every destination length is defined.

**`Exit`.** `Exit` requires `0 <= status <= 255`, the range a POSIX parent can observe; a violation is `assertion failed`. It calls C `exit`, which flushes buffered standard output and open file streams, so output written before `Exit` is never lost. Module bodies not yet run do not run. A program that never calls `Exit` still ends with status 0, and language failures still end with status 1.

**Passing the arguments.** `$main` takes `w %argc, l %argv` and passes both to `oberon_init(int argc, char **argv)`, which stores them in `oberon_argc` and `oberon_argv`. `runtime/standard.c` declares those two as `extern` and reads them; no other code touches them.

**Private interface.** `ProgramArgCount(): INTEGER` returns `argc - 1`, or 0 when `argc` is 0. `ProgramArg(n: INTEGER; VAR arg: ARRAY OF CHAR): INTEGER` copies and returns `res`. `ProgramExit(status: INTEGER)` calls `exit`. The range checks stay in `lib/Program.Mod` as `ASSERT`s, as in `Strings`.

## Test plan

- New `tests/corpus.rs` test `program_arguments_and_exit_status`: write a `Program`-importing module to a fresh temporary directory, compile it with the shared `compile` helper, and run `build/<name>` with the arguments `one`, an empty string, and `a longer argument`. The program prints `count`; prints each argument read into `ARRAY 32 OF CHAR` with its `res`; reads `a longer argument` into `ARRAY 4 OF CHAR` and prints `a l` with `res` 14; reads it into `ARRAY 0 OF CHAR` and prints `res` 17; prints `before exit` without a line feed; calls `Exit(3)`; then prints `unreached`. Check stdout exactly, empty stderr, and exit code 3.
- `tests/failures/ProgramArgRange.Mod`: `Program.Arg(0, ...)` with no arguments fails with `assertion failed`.
- `tests/failures/ProgramExitRange.Mod`: `Program.Exit(256)` fails with `assertion failed`.
- The existing corpus still passes, confirming the default status 0 and the failure status 1.

## Implementation plan

1. `runtime/oberon.c`: define `oberon_argc` and `oberon_argv`, change `oberon_init` to take and store `argc` and `argv`, and extend its comment.
2. `src/qbe.rs`: emit `export function w $main(w %argc, l %argv)` and `call $oberon_init(w %argc, l %argv)`.
3. `runtime/standard.c`: add the `extern` declarations and `oberon_lib_program_arg_count`, `oberon_lib_program_arg`, and `oberon_lib_program_exit`.
4. `src/sema/symbols.rs`: add `ProgramArgCount`, `ProgramArg`, and `ProgramExit` to `runtime_interface`.
5. `lib/Program.Mod`: declare `count*`, set it in the module body from `OberonRuntime.ProgramArgCount()`, and implement `Arg` and `Exit` with their `ASSERT`s before the native call.
6. `tests/corpus.rs`: add `program_arguments_and_exit_status`. Add the two `tests/failures` cases with their `.expected` files.

## Documentation updates

- `docs/standard-library.md`: add `Program` to the opening list and a `Program` section stating the interface, argument numbering, `Arg` truncation and `res`, the `Exit` range and flushing, and the default and failure statuses.
- `agents/roadmap.md`: remove the "Command line and exit status" item.
