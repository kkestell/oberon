# Runtime failure positions and output ordering review

## Scope and coverage

Commit `3a9d6b8`, "Report source positions for runtime failures and flush stdout
first": the runtime failure ABI in `runtime/oberon.c`, site representation in
`src/ir.rs`, every semantic lowering site in `src/sema/mod.rs`, QBE call
emission in `src/qbe.rs`, the failure corpus and output-order regression in
`tests/corpus.rs`, and the associated plan, work log, documentation, and
expected-output updates.

The review traced all runtime checks and traps from their source positions
through IR and QBE into the C runtime. Testing was limited to macOS ARM64; the
runtime ABI was not exercised on Linux or x86-64.

## Fixed

None.

## Findings

No open findings.

## Checks run

- `make test`: 43 unit tests and 6 integration tests pass, including all
  runtime-failure cases and `failure_follows_earlier_output`.
- `cargo fmt --check`: clean.
- `cargo clippy --all-targets -- -D warnings`: clean.
- `cc -O2 -Ivendor/bdwgc/include -Wall -Wextra -Werror -c runtime/oberon.c`:
  clean.
- `git diff --check 3a9d6b8^ 3a9d6b8`: clean.

## Verdict

The change consistently carries the selected source position through the
compiler, flushes standard output before reporting each language failure, and
preserves the existing failure messages and exit status. No findings remain.
