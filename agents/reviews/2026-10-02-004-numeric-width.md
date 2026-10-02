# Review: 64-bit numeric widths

## Scope and coverage

Reviewed commit `b9d227b` and the compiler, QBE lowering, runtime, bundled
modules, and corpus tests that depend on its widened `INTEGER`, `SET`, and
`REAL` representations. Traced scalar storage and calling conventions through
the Rust IR, generated QBE, C runtime entry points, file encodings, and the
standard-library wrappers. Validation ran on the supported amd64 SysV target;
other targets and impractically large output widths were not exercised.

## Fixed

- **PACK wraps exponents outside the C `int` range**
  (`runtime/oberon.c:185`): Widening `INTEGER` made exponents such as
  2147483648 legal, but narrowing them directly to the `int` accepted by
  `ldexp` could reverse their sign and underflow instead of overflow (or the
  reverse). Clamp only the library exponent to `INT_MIN..INT_MAX`, which gives
  the same binary64 overflow or underflow as multiplying by the requested
  power of two, and cover both boundaries in `RealPacking`.
- **Comments retain the obsolete 32-bit REAL and INTEGER model**
  (`src/driver.rs:82`, `src/parser.rs:744`): The comments described C float
  operations and named the old minimum integer after the implementation had
  moved to binary64 and 64-bit integers. Describe the C double operations and
  the width-independent `MIN(INTEGER)` case instead.

- **64-bit output widths are narrowed to C `int`**
  (`runtime/standard.c:23`, `runtime/standard.c:30`): `Out.Int` and `Out.Real`
  accept an `INTEGER` width, but each cast it to the `int` field width that
  `printf` consumes, so a width above `INT_MAX` truncated or changed sign.
  Format the value without padding and write the leading spaces in a loop
  over the `int64_t` width, so every positive width is a minimum width as
  the library documentation says.

## Findings

None remaining.

## Checks run

- `make test` — 43 unit tests and 6 integration tests passed before and after
  the fixes.
- `make` followed by compiling and running `tests/corpus/RealPacking.Mod` — the
  new large-exponent overflow and underflow regression passed.
- `cargo fmt --check` — passed.
- `cargo clippy --all-targets -- -D warnings` — passed.
- `cc -Wall -Wextra -Wconversion -fsyntax-only` on both runtime C files —
  passed.
- `git diff --check` — passed.

## Verdict

The numeric representations and their compiler/runtime boundaries are
consistent. The review fixed the one reachable numeric regression and two
obsolete comments; output formatting still needs an explicit policy or a
64-bit padding implementation for widths above `INT_MAX`.
