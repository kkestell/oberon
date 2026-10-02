# Review: Records and structured parameters

Reviewed against the uncommitted working tree on top of commit `f749b36` (`add chars bytes and strings`). The implementation follows [the Slice 12 plan](../plans/2026-08-02-012-records-and-structured-parameters.md).

## Verdict

The implementation covers the slice's record syntax, identity, layout, field visibility, field addressing, whole-record copying, structured parameters, module interfaces, IR, QBE, diagnostics, and corpus requirements. One low-priority finding was confirmed and fixed. No findings remain.

## Finding

### Low: The constant-expression checker accepts record equality as a well-typed relation

`Analyzer::lower_relation` correctly rejects every record operand because a record has no scalar value. `Analyzer::check_const_expr` uses a broader rule for `=` and `#`: it accepts any pair whose semantic types are identical. That includes records, even though Report 8.2.4 does not define equality for records and the slice plan requires records to be rejected in every value context.

This disagreement is reachable from a procedure-local constant declaration, because a procedure's constant declarations can name a module variable. The following program confirmed it:

```oberon
MODULE RecordConstRelation;
  TYPE R = RECORD n: INTEGER END;
  VAR r: R;
  PROCEDURE P;
    CONST Equal = r = r;
  END P;
END RecordConstRelation.
```

Compiling the program exited unsuccessfully and wrote only:

```text
review-probes/RecordConstRelation.Mod:5:19: 'r' is not a constant
Error: 1 error(s)
```

The compiler rejects the module, but it never diagnoses that `r = r` is not a legal relation. The constant-expression type walk should apply the same equality operand rules as executable lowering before constant evaluation reports that the operands are not constants. A regression should place record equality in a procedure-local constant declaration so this second semantic path stays covered.

### Resolution

`Analyzer::check_const_expr` now accepts identical equality operands only when they have scalar types, while retaining the separate character and string relation rule. Identical record and non-character-array types receive a direct operator diagnostic. `RecordConstRelation` preserves the procedure-local constant path and now reports `operator '=' is not defined for R operands` without the misleading nonconstant-variable diagnostic.

## Verified implementation

Each `RECORD` constructor builds one shared descriptor. Direct type declarations stamp the descriptor with their source name, aliases retain the original descriptor, and imported interfaces clone the shared handle. Separately constructed records remain distinct even when their fields or printed names match.

Record layout uses declaration order and per-type alignment. `RecordLayout.ssa` places a `CHAR` followed by an `INTEGER` at offsets zero and four, reserves eight bytes, and copies eight bytes. Its mixed record places a three-byte BYTE array between REAL fields at offsets four and eight, reserves twelve bytes, and copies twelve bytes. Globals carry the computed alignment, and local records use `alloc4` with their exact sizes.

Empty records have size zero and alignment one. The zero-length-array record in `Records.ssa` has alignment four and size four, and its whole-record copy passes four to `oberon_copy`. Empty-record assignment still emits a zero-byte copy after resolving both designators.

Field selection emits one pointer addition for each selector, including offset zero. Nested record and array selectors chain those additions with the existing checked index operation. `FieldIndexBounds` reaches the existing runtime check through a record field and exits with `array index out of bounds`.

Whole-record assignment uses `oberon_copy`, preserves padded bytes, accepts self-assignment and overlap through the runtime's `memmove`, and leaves the destination independent of later source mutation. Character-array fields retain string assignment and bounded comparison behavior.

Structured value and `VAR` parameters both use one QBE pointer-class argument. Structured value formals have no callee-side copy and remain read-only through every field and index selector. Structured `VAR` formals remain writable. Scalar value parameters still receive a value-class argument and copy it into a local slot. `ParamAlias` prints `4  41` and `5  42`, confirming that mutation through the `VAR` alias is observable through the structured value parameter.

The module gates preserve record identity and field visibility across imports. Marked fields of exported and private record types are selectable by clients. Unmarked fields are reported as absent. Imported record variables, their fields, and their array elements remain read-only, while the same imported variables can be passed to structured value parameters.

A targeted program additionally confirmed constant `LEN` through nested record fields, an imported structured value forwarded through another value parameter, a nested procedure taking a structured value parameter, a dynamic index through a record field, and assignment between elements of an array of empty records. It exited successfully and printed `3  20  20` with empty standard error.

## Declined changes

The review does not request record extension, extension-compatible assignment or parameters, type tests, type guards, the record form of `CASE`, pointers, open arrays, or procedure types. Those remain assigned to later slices.

The review does not request copying structured value actuals into callee frames. The read-only reference convention and its observable aliasing behavior are deliberate decisions in the slice plan and agree with the consulted reference compilers.

The review does not request fixed-array string actuals. The planned rule rejects them and leaves string parameter passing to open arrays of CHAR.

## Verification

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `git diff --check` pass. The test run contains 35 unit tests and two integration tests.

`ParamAlias`, `RecordLayout`, `RecordParams`, `RecordTable`, `Records`, and the cross-module `RecordApi` program were compiled and run directly. Each exited successfully, wrote empty standard error, and produced byte-for-byte checked-in standard output.

The generated IL for every pre-slice corpus root was compared with IL produced from commit `f749b36` in a separate archive. No file differed. `Params.ssa` retained SHA-256 `bc6627e4199e252930401829769ea56477ae76987738f442f8111d8121233291`.

`RecordLayout.ssa`, `RecordParams.ssa`, `Records.ssa`, and `ParamAlias.ssa` were inspected for field offsets, padded sizes, global alignment, local allocation, whole-copy byte counts, structured pointer arguments, the absence of structured callee copies, scalar parameter slots, and the planned aliasing behavior.
