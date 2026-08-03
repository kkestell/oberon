# Review: REAL values and numeric conversions

Reviewed against the uncommitted working tree on top of commit `5cd7b14` (`set values and operations`). The implementation follows [the Slice 9 plan](../plans/2026-08-02-009-real-values-and-numeric-conversions.md).

The untracked fixed-arrays plan is the proposed Slice 10 work. It treats this REAL implementation as its baseline, but its array design is outside this review.

## Verdict

The slice implements the complete `REAL` basic type described by its plan. The lexer, semantic types, constant evaluator, module interfaces, IR, QBE emitter, C runtime, and procedure calling convention agree on IEEE 754 binary32 values.

One medium-priority finding was confirmed and fixed. No findings remain.

## Finding

### Constant `FLOOR` arguments are not range-checked in executable statements

The plan distinguishes constant and dynamic failures of `FLOOR`. A constant argument outside the result domain must be a source diagnostic. Only a dynamic argument should reach `oberon_floor` and fail at run time.

`Analyzer::eval_const_builtin` enforces the range while folding a constant declaration. `Analyzer::lower_builtin` always emits the runtime call after checking only the argument type. It does not ask whether the argument is a constant whose value is already known to be outside the domain.

This program confirmed the gap:

```oberon
MODULE ReviewFloorStatic;
  IMPORT Out;
BEGIN
  Out.Int(FLOOR(3.0E9), 1)
END ReviewFloorStatic.
```

Compiling it exited with status zero. Both compiler output streams were empty:

```text
target/debug/oberon build/ReviewFloorStatic.Mod
```

Running the generated executable exited with status one and wrote:

```text
FLOOR result is outside INTEGER range
```

The compiler should reject this program at the `FLOOR` argument. The same gap applies to a constant infinity, a constant NaN, and the upper endpoint reached through `FLT(2147483647)` when those expressions appear outside a constant declaration.

The lowering path should perform the same optional constant-domain check before emitting `oberon_floor`. The range and finiteness rule should remain shared with constant evaluation so the two paths cannot drift. A regression in `tests/errors/` should put an out-of-domain constant `FLOOR` call in a module statement, because `FloorConstRange.Mod` currently exercises only constant declarations.

### Resolution

`Analyzer::check_floor_argument` now uses optional constant evaluation before emitting the runtime call. A dynamic argument retains the runtime check. A known valid constant retains the ordinary runtime lowering, while a known constant outside the domain produces the same source diagnostic as constant folding.

The range and conversion live in the shared `floor_const` helper. Both constant declarations and executable expressions therefore use the same finiteness checks and endpoints.

`FloorConstRange.Mod` now includes `n := FLOOR(3.0E9)` in its module body. The compiler rejects it with this diagnostic:

```text
tests/errors/FloorConstRange.Mod:15:14: constant FLOOR result is outside INTEGER range
```

## Verified implementation

REAL literals are parsed directly to `f32`. Overflowing literals are diagnosed, underflow rounds to zero, scale factors with either sign work, and `1..2` remains an integer followed by a range delimiter.

`Type::Real`, `ConstValue::Real`, `ir::Ty::Real`, and `ir::Value::Real` preserve the distinction from every word-class scalar. REAL globals and locals reserve four bytes. REAL values use QBE class `s`, while REAL variable parameters use pointer class `l`.

Unary plus and minus, arithmetic, quotient, equality, inequality, and all four ordering relations select their operation from the operand type. INTEGER and REAL never mix implicitly. SET overloads and existing INTEGER and BOOLEAN behavior remain intact.

Folded REAL arithmetic uses `f32` at each source operator. The `Reals` and `RealIeee` programs compare folded results with runtime results. They cover ordinary arithmetic, signed zero, overflow, division by zero, infinity, NaN equality, NaN inequality, and NaN ordering.

QBE immediates use the signed spelling of the binary32 bit pattern. Generated IL uses `s` arithmetic, `s` negation, and the six single-precision comparison operations. `FLT` emits `swtof`. `ABS` and `FLOOR` call C wrappers with `s` arguments and the correct result classes.

REAL works as a module variable, local variable, constant, value parameter, variable parameter, proper-procedure argument, and function result. The cross-module program shares the same four-byte exported global and uses `s` loads and calls on both sides of the module boundary.

`PACK` passes a REAL address followed by one INTEGER value. `UNPK` passes a REAL address and an INTEGER address. Positive, negative, subnormal, zero, infinite, and NaN paths agree with the plan's documented choices. The PACK exponent function is called once.

All six positive REAL binaries exited successfully. Their standard error streams were empty, and their standard output matched the checked-in expected files. The four FLOOR failure binaries and two UNPK failure binaries compiled successfully, exited unsuccessfully at run time, and matched their expected standard error exactly.

The generated `Params.ssa` still has SHA-256 `bc6627e4199e252930401829769ea56477ae76987738f442f8111d8121233291`. This matches the hash recorded before the typed REAL changes and confirms that the representative word-class IL remained byte-for-byte stable.

## Declined changes

The review does not request binary64 REAL values, a configurable floating-point representation, or a target-description layer. Binary32 and the native QBE `s` class are deliberate choices in the plan.

The review does not request direct QBE lowering for the runtime-backed operations. The small C wrappers supply the required checked conversion and library behavior without expanding the backend.

The review does not request an optimization pass that folds valid executable `FLOOR` calls. The finding concerns only deterministic invalid constants that the plan says must be rejected at compile time.

The review does not cover the fixed-arrays plan. No fixed-array implementation is present in this working tree.

## Verification

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `git diff --check` pass. The test run contains eighteen unit tests and two integration tests.

`Reals`, `RealBuiltins`, `RealProcedures`, `RealPacking`, `RealIeee`, and `RealApi` were compiled and run directly. Their output matched the checked-in expectations.

`build/Reals.ssa`, `build/RealBuiltins.ssa`, `build/RealProcedures.ssa`, `build/RealPacking.ssa`, `build/RealIeee.ssa`, and `build/RealApi.ssa` were inspected for storage size, calling classes, exact immediates, arithmetic, comparisons, conversion, and runtime calls.
