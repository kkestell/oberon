# Review: lexer, parser, sema, and integer codegen

Reviewed the implementation of plans
[001](../plans/2026-08-02-001-end-to-end-qbe-libgc-integration.md) and
[002](../plans/2026-08-02-002-walking-skeleton-lexer-to-codegen.md), covering
`lexer.rs`, `parser.rs`, `ast.rs`, `sema.rs`, `qbe.rs`, `diag.rs`, `driver.rs`,
the C runtime, and the corpus harness.

The tree reviewed was the uncommitted working copy that later became commit
`4010aeb`. Every finding below was fixed before that commit landed, so the line
numbers from the original review would not resolve against anything in history
and have been replaced with function names.

Two of the five findings were confirmed by compiling and running a program, not
by reading alone. Those repro cases are recorded here because they are what
makes the findings checkable later.

## What held up

The stage seams match the architecture section of AGENTS.md, and each file stays
on one concern. Comments cite the Report where behaviour is non-obvious instead
of restating the code.

Two things were checked against the vendored reference compilers and found
correct. The number scanner consumes digits and `A`–`F` greedily and only then
decides between `H`, `X`, and `.`, which is what `ORS.Mod`'s `Number` does — so
`10DIV` failing to scan is Wirth's behaviour, not a bug. And `ScaleFactor`
accepting only `E` matches
[oberon07-grammar.ebnf](../../references/oberon07-grammar.ebnf) line 10, even
though Wirth's own scanner also accepts `D`.

Operator precedence is right, including `-a*b` parsing as `-(a*b)` because unary
minus takes a whole term.

## 1. Constant folding and generated code disagreed about DIV and MOD

`eval_const` in `sema.rs` folded with `checked_div_euclid` and
`checked_rem_euclid`. The `Binary` arm of `Gen::expr` in `qbe.rs` emitted QBE's
`div` and `rem`, which truncate toward zero. The same expression therefore gave
two different answers depending on whether it was a constant.

```oberon
CONST CD = (-7) DIV 2; CM = (-7) MOD 2;   (* folded to -4 and 1 *)
x := -7; d := x DIV 2; m := x MOD 2;      (* computed -3 and -1 *)
```

Running that printed `-41` on the first line and `-3-1` on the second.

The folded answer was the correct one. Report 8.2.2 defines `x = q*y + r` with
`0 <= r < y`, and `-1` is not in that range. So codegen was the wrong side. A
`// TODO` in `qbe.rs` already noted that `div` and `rem` truncate, but it read as
a deferred nicety rather than a live disagreement between two stages, which is a
worse state than being uniformly wrong.

**Fix.** Both stages now compute the floored remainder with the same expression,
`((x REM y) + y) REM y`, and derive the quotient by exact division. OBNC's
`OBNC_MOD` uses that same formula. The formula is branchless, which mattered
because `Gen` emitted straight-line code and had no label machinery yet — the
obvious sign-test version would have forced basic blocks into codegen before
`IF` and `WHILE` needed them. The comment in each file points at the other so
the two cannot drift apart silently.

## 2. Sema never checked for duplicate declarations

Every insertion into the scope map in `analyze` was a bare `HashMap::insert`, so
a redeclaration silently overwrote. `VAR K, K: INTEGER;` compiled clean and
emitted the allocation twice:

```
%K =l alloc4 4
%K =l alloc4 4
```

QBE accepted that, but defining the same temporary twice is not something to
lean on.

The sharper version involved imports. A variable named `Out` overwrote the
`Symbol::Module` entry that `IMPORT Out` installed. The later `Out.Int(...)`
then resolved `Out` to a plain variable, fell through the match in `resolve`,
and hit `todo!("record field selection")`. That is a panic and exit code 101 on
source the compiler should have rejected with a diagnostic, which is exactly the
boundary-sanity rule in AGENTS.md.

**Fix.** A `declare` helper checks before inserting and every declaration path
goes through it. First declaration wins, so the rest of the module still checks
against something sensible. Report 4: "No identifier may denote more than one
object within a given scope."

## 3. Hex literals could not express a 32-bit bit pattern

`x := 0FFFFFFFFH` was rejected with "integer literal out of range". The lexer
parsed hex into `i64` and sema then range-checked against `i32`.

Wirth does something different. `ORS.Mod`'s `Number` accumulates
`k := k*10H + h` into a 32-bit `LONGINT` and marks the loop
`(*no overflow check*)`, so `0FFFFFFFFH` wraps to `-1`. That matters because bit
patterns are most of what hex literals are for.

**Fix.** The lexer parses through `u32` and casts, so a hex literal is a 32-bit
pattern. A related gap was fixed at the same time: `-2147483648` was rejected
because the literal was range-checked before the unary minus was applied. The
parser now folds a minus sign directly into an integer literal. Only a bare
literal folds, so `-a*b` still means `-(a*b)`.

## 4. Some `todo!()` calls sat on valid source

AGENTS.md sanctions `todo!()` for the unwritten half of the language, and most
of them were fine. But there is a distinction worth drawing. A `todo!()` that
only a future stage can reach costs nothing. A `todo!()` that today's user
reaches by writing correct Oberon turns a missing feature into a panic.
`VAR x*: INTEGER;` is legal Oberon-07 and exited 101 with a backtrace note.

Returning a diagnostic is the same amount of code as `todo!()`, and it means the
corpus harness prints a file and line from the user's module instead of a
location inside the parser.

**Fix.** An `unsupported` helper in `parser.rs` covers export marks, `TYPE` and
`PROCEDURE` declarations, structured statements, index and dereference
selectors, relations, `OR`, real division, `&`, function calls in expressions,
and non-INTEGER factors. The one in `resolve` is handled separately, and
mattered most because finding 2 made it reachable from a program that should
have produced a diagnostic instead.

## 5. Lexer errors cascaded into invented parse errors

`driver::build` lexed, then parsed, and only then reported. When the lexer
rejected a token it returned `None` and the token was dropped, so the parser ran
on a stream with a hole in it and invented a second error. Both were printed:

```
Bad.Mod:4:10: illegal character '@'
Bad.Mod:4:12: expected 'END', found Int(2)
```

**Fix.** The driver reports and bails after lexing.

## Smaller notes

- The `cannot assign to` and `is not a procedure` messages printed only the base
  identifier, so `Out.Int := 5` reported just `Out`. Fixed with a
  `Designator::name` helper that renders the qualified name.
- The declaration loop in `module()` accepts `CONST` and `VAR` sections repeated
  and interleaved, where the grammar allows at most one of each in a fixed
  order. Harmless leniency and probably right while the shape is moving, but it
  is a deliberate divergence rather than an accident, so it earned a comment
  saying so.
- `expect` formats the unexpected token with `{:?}`, so users see
  `found Int(2)` and `found End`. A `Display` impl for `Tok` would improve every
  parse error, but it is a sixty-line mechanical match for a cosmetic win.
  Deliberately not done.
- `Gen` owns the scope by value and clones the resolved symbol to get around the
  borrow checker. Fine as-is; the typed IR will delete it.

## Test harness

The three findings that produce diagnostics rather than output had nowhere to
live, so `tests/corpus.rs` grew a second corpus. `tests/corpus/` compiles and
runs a module and compares stdout. `tests/errors/` expects compilation to fail
and compares stderr. Modules are passed as paths relative to the repo root so
the positions in the expected files are not tied to a checkout location.

Regression modules added: `DivMod.Mod` (computes six quotients and remainders
twice, once folded and once at runtime, and prints two lines that must be
byte-identical), `HexLit.Mod`, `Duplicate.Mod`, `ExportMark.Mod`, and
`IllegalChar.Mod`.

The harness was checked for vacuous passing by corrupting one expectation of
each kind and confirming both failed.

## Left alone

`cargo fmt --check` reports diffs across the whole tree, including untouched
files, so the project is not on default rustfmt and nothing was reformatted.

`x DIV 0` still faults at runtime rather than reporting an Oberon runtime error.
The constant case is caught, but the variable case reaches a hardware divide.
That is the runtime-checks work AGENTS.md lists as still to come.
