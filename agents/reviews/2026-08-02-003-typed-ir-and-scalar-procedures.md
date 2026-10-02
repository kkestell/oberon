# Review: Typed IR and scalar procedures

Reviewed against the uncommitted tree on top of commit `7eb9384` ("agents"),
implementing the plan in
[../plans/2026-08-02-004-typed-ir-and-scalar-procedures.md](../plans/2026-08-02-004-typed-ir-and-scalar-procedures.md).

## Verdict

The slice is complete against its plan and the architecture goal is achieved:
qbe.rs is now a 197-line printer over `ir::Program` with no `Scope`, no
`resolve`, and no `ast::` import, and every semantic decision — name
resolution, storage, short-circuiting, floored division, the zero check —
lives in sema.rs. All four gate programs produce correct output, the failure
corpus traps as specified, and `cargo fmt`, `cargo clippy -- -D warnings`, and
`cargo test` are all clean. One real defect was found: the floored MOD/DIV
instruction sequence overflows for operands near the INTEGER limits, producing
a wrong (negative) MOD result at runtime and a spurious "overflows" diagnostic
when the same expression is folded. It is an extreme edge case, inherited from
the formula the plan deliberately carried over from the old backend, but it
violates Report §8.2.2 and it makes folding and runtime disagree — the exact
property this slice set out to pin. Recorded below with a reproducing program;
the fix was applied in the same commit that lands this review, so the code as
committed no longer contains the overflowing formula.

## Verified behavior

**The gate programs.** Compiled and run by hand, per the plan's verification
step 5:

- `Fib` prints `55` (Fib(10), direct recursion).
- `Swap` prints `2 1 0` then `9 4 1` — module variables and locals both
  swapped through `VAR` parameters, the BOOLEAN flag flipped through its own
  `VAR` parameter.
- `Counter` prints `42` — module data written by one procedure and read by
  another, so it outlives both activations.
- `Params` prints `10 1 11 11 0`, then `-4 -4 1 1`, then `2`. The second line
  is the load-bearing one: `-7 DIV 2` and `-7 MOD 2` computed once by constant
  folding and once through the emitted instruction sequence, side by side,
  agreeing on the floored answers `-4` and `1`.
- `DivZero` prints `DIV or MOD by zero` on stderr and exits with status 1.

**The generated IL**, per verification step 3. `build/Counter.ssa` declares
`data $Counter.count = align 4 { z 4 }` and both procedures reach it by
symbol; the initializer is `function $.Counter.init()` and `$main` calls it
after `$oberon_init`. `build/Swap.ssa` shows the three `VAR` parameters as
`l` temps used directly as store targets, while the local `temp` gets an
`alloc4 4` slot. `Counter.Bump` shows the §10.1 value-parameter contract: a
slot plus an entry store of the incoming argument. `build/Params.ssa` contains
per-site `div.zero` trap blocks ending in `hlt` for its variable divisors; a
hand-compiled module dividing by the literal `2` contains none, confirming the
literal-divisor skip.

**VAR parameters forward.** The corpus passes module variables and locals as
`VAR` actuals but never a `VAR` parameter itself, which is the one case where
the actual's address is already a pointer temp (`Addr::Temp`) rather than a
slot or a global. Verified by hand:

```oberon
MODULE Forward;
IMPORT Out;
VAR a: INTEGER;
PROCEDURE Inner(VAR x: INTEGER);
BEGIN x := x + 1 END Inner;
PROCEDURE Outer(VAR x: INTEGER);
BEGIN Inner(x); Inner(x) END Outer;
BEGIN
  a := 5; Outer(a); Out.Int(a, 0); Out.Ln
END Forward.
```

prints `7`: the pointer passes through two call layers and both increments
land on the module variable.

**The function/proper distinction, both directions in one module.** A module
containing `x := P()` (proper procedure in an expression) and a bare `F`
statement (function procedure as a statement) reports exactly
`'P' cannot be used as a value` and
`function 'F' cannot be called as a statement`, one diagnostic each.

**Mutual recursion is an ordinary scope error.** The plan records that
module-level mutual recursion is inexpressible; a module where `A` calls the
later-declared `B` reports `undeclared identifier 'B'` at the call, as
predicted.

**Duplicate parameter names do not panic.** `PROCEDURE P(x: INTEGER; x:
INTEGER)` reports `'x' is already declared` once and compilation fails
cleanly, even though the IR builder had already registered both incoming
parameter temps — the poisoning rule (any diagnostic discards the whole
program) covers the half-built procedure.

## Findings

### The floored MOD/DIV sequence overflows near the INTEGER limits

`floored_mod` in sema.rs emits `((x rem y) + y) rem y`, and constant folding
uses the same formula with checked arithmetic, exactly as the plan specified.
The formula is wrong when `rem + y` exceeds MAX(INTEGER), which happens for
large positive divisors. Confirmed at runtime:

```oberon
MODULE ModEdge;
IMPORT Out;
VAR x, y: INTEGER;
BEGIN
  x := 2147483646; y := 2147483647;
  Out.Int(x MOD y, 0); Out.Ln
END ModEdge.
```

prints `-3`. The correct answer is `2147483646` (x is smaller than y, so x MOD
y is x), and §8.2.2 requires `0 <= x MOD y < y` for a positive divisor, so a
negative result is a Report violation, not merely an odd value. The wrap also
poisons the derived quotient: the same operands under DIV give `-1` instead
of `0`.

The DIV lowering had a second overflow of its own. It computed the quotient as
`(x - mod) / y`, and the subtraction leaves INTEGER when the dividend is
MIN(INTEGER): for `Min DIV 3` the remainder adjusts to `1`, `Min - 1` wraps to
MAX(INTEGER), and the division then returns a large positive number instead of
`-715827883`.

The folded path fails differently. `CONST K = 2147483646 MOD 2147483647;` is
rejected with `constant expression overflows`, although the value of K is
representable. So the slice's pinned property — folding and runtime agree
because they share one formula — holds only formally: they share the formula,
and the formula's intermediate step overflows, at which point checked and
wrapping arithmetic diverge (a diagnostic on one side, a wrong number on the
other).

This is inherited: the old qbe.rs emitted the same sequence, and the plan
explicitly carried it over. It is not a regression, and no realistic program
divides by numbers within one of MAX(INTEGER). But it was the only place
where the compiler computes an integer operation the Report defines and gets
a different answer, so it was fixed in the same commit that lands this
review. The truncated results are off by one step exactly when the remainder
is nonzero and its sign differs from the divisor's. `floor_adjust` in sema.rs
now computes that condition as a 0-or-1 value from the comparisons the IR
already has (no new instructions were added to the IR): MOD adds
`adjust * y` to the truncated remainder, and DIV subtracts `adjust` from the
truncated quotient, so no intermediate can leave INTEGER. Constant folding
applies the identical adjustment with checked arithmetic. New rows in
`DivMod.Mod` pin the limit cases — `2147483646` DIV and MOD `2147483647`,
MIN(INTEGER) by `3`, and MIN(INTEGER) by MAX(INTEGER) — each computed once by
folding and once at runtime, and the two output lines agree.

One boundary stays undefined on purpose: `MIN(INTEGER) DIV -1` has no
representable result. Folding rejects it as an overflow; at runtime the
hardware divide traps. The Report's `0 <= r < y` constraint only defines DIV
and MOD for positive divisors anyway, so nothing is pinned about negative
ones beyond what the existing corpus rows already showed.

### The corpus class never checks exit status (minor, accepted)

`tests/corpus.rs` compares a corpus binary's stdout but ignores its exit
status. A program that printed everything expected and then crashed — say a
trap reached after the last `Out.Ln` — would pass. The errors and failures
classes both pin their statuses; the corpus class could cheaply assert
success next time the file is edited. Noted, not blocking: today every corpus
binary exits 0 by construction of the `$main` stub.

### Procedures with unresolvable signatures cascade (minor, accepted)

When a formal type or the result type fails to resolve, the procedure symbol
is never declared, so every later call adds `undeclared identifier`. This is
the same accepted cascade recorded for constants in the previous review
(2026-08-02-002): the follow-on error points at real code, and a
poisoned-symbol sentinel is still more machinery than the problem deserves.
Recorded so the growing pattern has a trail.

### Plan deviations, all benign

- The plan sketched `check_expr` keeping its name; the implementation renames
  it `lower_expr` and adds `lower_condition`, `lower_call`, `var_actual`, and
  `addr_of` alongside. The names describe what the functions now do.
- The plan's IR sketch had no `check_const_expr`/`check_const_call` pair; the
  implementation keeps the existing two-step constant scheme (type-check the
  whole expression, then fold) and extends `check_const_call` so a procedure
  call inside a constant expression gets argument checking before `eval_const`
  rejects it with `constant expression contains a procedure call`. Strictly
  more diagnostics than the plan promised.
- `Symbol::Proc` for the `Out` built-ins moved into an `out_scope()` helper;
  same shape as user procedures, as the plan required.

## What was checked and found sound

- The IR matches the plan's sketch almost token for token: flat `Vec<Inst>`,
  `Value`/`Addr` split, `Arg::Val` vs `Arg::Ref` carrying the `w`/`l`
  distinction, `Param { temp, pass }`. ir.rs is data only, no logic.
- The label-after-terminator invariant is asserted in `emit_proc`, and label
  and temp counters are per-procedure (`ProcBuilder`), so names like
  `@.if.end0` are function-scoped, which QBE requires.
- Mangling follows the plan: `$Mod.name` for user symbols, `$.Mod.init` for
  the initializer. A module named `Fib` containing a procedure named `Fib`
  compiles (module names never enter the scope, and `Fib.Fib` cannot collide).
- Scopes are a stack; `declare` checks only the innermost map, so locals and
  parameters legally shadow module names, and first-declaration-wins is
  preserved (the `Duplicate` error module still pins three errors).
- Arguments evaluate left to right; a `VAR` actual must be a designator
  resolving to `Symbol::Var` with an identical type. Constants and
  expressions in `VAR` position are diagnosed per argument
  (`VarActualExpr` pins two errors in one call).
- All five new RETURN/call diagnostics are pinned by error modules with exact
  positions, plus `NestedProc` pinning the parser's clean "not yet supported"
  message.
- `lower_return` handles all combinations of declared result type, resolved
  result type, and RETURN clause presence; every poisoned path still emits a
  `Ret` so the builder finishes, relying on the discard-on-any-diagnostic
  rule the plan states.
- The zero check guards the shared divisor before the floored sequence for
  both DIV and MOD, per site, skipped only for nonzero integer literals; a
  literal zero divisor compiles to an unconditional trap, as the plan chose.
- The runtime addition is exactly the planned six lines; `oberon_div_by_zero`
  writes the message the failure corpus pins and exits 1.
- The failures harness class requires compile success, nonzero exit, and
  byte-exact stderr, and stdout is deliberately not compared, as planned.
