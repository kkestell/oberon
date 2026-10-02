# Program arguments and exit status review

## Scope and coverage

Commit `11ccbbc`, "Add the Program module for arguments and exit status": `lib/Program.Mod`, `runtime/oberon.c`, `runtime/standard.c`, `src/qbe.rs`, `src/sema/symbols.rs`, the new test in `tests/corpus.rs`, the two `tests/failures` cases, and the `docs/standard-library.md` and roadmap updates, read against `agents/plans/2026-10-02-020-program-arguments-and-exit-status.md`. Also checked the other documents that list the bundled modules (`bugs/prompt.md`, `bugs/prompt-volume.md`).

Tested on macOS ARM64 only.

## Fixed

- **Command-line comment attached to the collector setup** (`runtime/oberon.c:10`): the sentence about keeping the command line was appended to the interior-pointer comment, and `oberon_argc` and `oberon_argv` were declared between that comment and `oberon_init`, so the collector explanation sat over the two globals instead of the function it describes. The globals now come first with their own comment, and the collector comment sits directly over `oberon_init` again.
- **Bug-hunt prompts omit `Program`** (`bugs/prompt.md:59`, `bugs/prompt-volume.md:71`): both prompts enumerate the bundled modules, and neither named `Program`, so a hunter following `prompt.md` would not know `docs/standard-library.md` specifies it, and `prompt-volume.md` did not list it among the modules to leave alone. Both lists now include `Program`.

## Findings

No open findings.

## Checks run

- `make test`, before and after the fixes: 43 unit and 5 corpus tests pass, including `program_arguments_and_exit_status`, `ProgramArgRange`, and `ProgramExitRange`.
- A client that assigns `Program.count`, passes it as a `VAR` argument, and passes it to `INC` is rejected with three read-only diagnostics.
- A program that writes a registered file through a rider, writes once more after `Register`, prints to stdout redirected to a file, and calls `Program.Exit(255)`: status 255, stdout holds the text, and the file holds both integers.
- A module whose imported module calls `Program.Exit(0)` from its body: status 0, and the client body does not run.

## Verdict

`Program` works as planned and documented. Both findings were comment and documentation fixes; nothing is open.
