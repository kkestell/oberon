# Review: SET values and operations

Reviewed against the uncommitted working tree on top of commit `34257d4`
(`real modules, exports, and initialization`). The implementation follows
[the Slice 8 plan](../plans/2026-08-02-008-set-values-and-operations.md).

The untracked Slice 9 plan is outside this review. No Slice 9 implementation
was present in the working tree.

## Verdict

The implementation covers the complete `SET` basic type described by the
slice. The parser, semantic types, constant values, module interfaces, IR,
QBE mapping, runtime trap, and storage and procedure roles agree with the
Report and the representation decisions in the plan.

One medium-priority finding was confirmed and fixed. No findings remain.

## Finding

### Invalid constant set elements are deferred to run time

`Analyzer::check_set_element` asks `eval_const` for an element value. It handles
an `Ok` result as a known constant. It handles every `Err` result as proof that
the expression is dynamic.

Those errors have two meanings. A variable or user procedure call makes the
expression nonconstant. Division by zero, arithmetic overflow, and a bad
constant shift make a constant expression invalid. The first case needs a
runtime bounds check. The second case needs a source diagnostic.

The Slice 8 plan calls out this distinction explicitly. The current helper
does not preserve it.

The following program confirmed the problem:

```oberon
MODULE ReviewSetConstError;
  VAR s: SET;
BEGIN
  s := {1 DIV 0};
  s := {2147483647 + 1}
END ReviewSetConstError.
```

This command exited with status zero. It wrote no standard output or standard
error:

```text
target/debug/oberon build/ReviewSetConstError.Mod
```

The generated executable then exited with status one and wrote:

```text
DIV or MOD by zero
```

The compiler should reject the first element as a constant division by zero.
It should not emit an executable. The same problem affects a range endpoint,
the left operand of `IN`, and the element argument of `INCL` or `EXCL`, because
all five positions use `check_set_element`.

The optional constant-evaluation path should represent three outcomes. They
are a value, a nonconstant expression, and an invalid constant expression.
The last outcome should retain its diagnostic. A regression should place an
invalid constant expression in one of the runtime contexts above. The existing
tests cover constants whose values are outside 0 through 31, but those
expressions evaluate successfully and do not exercise this distinction.

### Resolution

`Analyzer::try_eval_const` now preserves all three outcomes. It returns a
folded value when evaluation succeeds. It returns no value when the expression
contains a variable or user procedure. It retains the diagnostic when an
expression made only from constants is invalid.

`SetElementConst.Mod` now uses `1 DIV 0` as a constructor element in a module
statement. The compiler rejects it with this diagnostic:

```text
tests/errors/SetElementConst.Mod:17:11: constant DIV or MOD by zero
```

## Verified implementation

`SET` is a distinct semantic type, constant value, and IR type. It shares the
four-byte QBE word representation with `INTEGER` and `BOOLEAN` without becoming
assignment-compatible with either one.

The parser retains unary plus for semantic checking. It accepts set
constructors, ranges, membership, and the slash operator at their Report
precedences. Semantic analysis rejects unsupported operand combinations.

Constant folding and runtime lowering agree for union, difference,
intersection, symmetric difference, complement, equality, inequality,
membership, and `ORD`. The positive `Sets` program compares the folded and
runtime forms. It exited successfully and printed the checked-in output,
including the values for bit 31 and the full set.

The constant and runtime range formulas produce the empty set for a reversed
range. Dynamic endpoints are evaluated once in source order. The `SetRanges`
program exited successfully and printed:

```text
            60   2    12
             0   2    12
   -2147483623   4  1122
```

The generated `SetRanges.ssa` calls each endpoint procedure once. Each call is
followed by signed comparisons against zero and 31 before its value reaches a
shift. The range mask uses `shl`, `shr`, and `and` with checked shift counts.

The implementation does not emit the endpoint comparison and branch proposed
by the plan for a dynamic reversed range. Its two masks have disjoint bits
when the low endpoint exceeds the high endpoint, so their intersection is
already zero. This is equivalent and smaller, so it is not a finding.

`INCL` and `EXCL` use the writable-designator path. They change module
variables, locals, basic value-parameter copies, and `VAR` parameters. They
reject constants, expressions, and imported variables. The positive procedure
program and the negative imported-variable module both pass their corpus
checks.

The cross-module program folds an exported `SET` constant. It reads an
exported `SET` variable. It passes `SET` values and addresses to exported
procedures. It also receives a `SET` function result. The program exited
successfully and printed the checked-in output.

The QBE output gives every `SET` value and result class `w`. A `SET` variable
parameter has class `l`. Locals and globals reserve four bytes. The exported
`SetSupport.flags` object has one four-byte data definition, and the support
and client procedures use that same symbol.

The runtime failures for a singleton, range endpoint, membership test,
`INCL`, and `EXCL` all exit unsuccessfully with this standard error:

```text
SET element out of range
```

## Declined changes

The review does not request a dedicated IR instruction for a set operation.
The existing word-sized bit operations express every operation directly.

The review does not request a configurable set limit or target description.
The documented 32-bit representation is the deliberate target choice for
this compiler.

The review does not request that the dynamic range lowering copy the plan's
branch shape. The implemented mask intersection has the same value and keeps
every shift count inside the checked domain.

The review does not cover the untracked Slice 9 plan. It contains no code and
does not change the Slice 8 result.

## Verification

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`,
and `git diff --check` pass. The test run contains fourteen unit tests and two
integration tests.

`Sets`, `SetRanges`, `SetProcedures`, and `SetApi` were also compiled and run
directly. Every compiler and program invocation exited with status zero. Every
standard error stream was empty. Each standard output matched its checked-in
expected file.

`build/Sets.ssa` contains `or`, `and`, and `xor` operations for the set
operators. Difference intersects the left operand with the complemented right
operand.

`build/SetProcedures.ssa` uses word values for `SET` parameters and results.
It uses an address for the `VAR` parameter. Its local `SET` slot reserves four
bytes.

The unchanged `Params` module still produces SHA-256
`bc6627e4199e252930401829769ea56477ae76987738f442f8111d8121233291`.
This is the hash recorded before Slice 8.
