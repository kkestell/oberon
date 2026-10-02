# Slice: REAL values and numeric conversions

## Context

After Slice 8 the compiler supports the INTEGER, BOOLEAN, and SET basic
types. Each type works in constants, module variables, procedure locals,
parameters, results, and module interfaces. The parser also accepts `/` for
SET symmetric difference and retains unary plus for semantic checking.

The lexer already recognizes the Report's decimal REAL syntax. It currently
stores a real token as an `f64`, while the parser still rejects it. Semantic
analysis has no REAL type or constant value. The IR and QBE emitter support
only word-class values. `ABS` accepts only INTEGER. `FLT`, `FLOOR`, `PACK`, and
`UNPK` are absent from the universe scope.

This slice adds the complete REAL basic type. It adds literals, constants,
arithmetic, relations, storage, procedure calls, and module interfaces. It
also adds the REAL form of `ABS`, the conversion functions `FLT` and `FLOOR`,
and the variable-modifying procedures `PACK` and `UNPK`.

Report sections implemented: section 3 for REAL literals and scale factors;
section 4 for the predefined identifiers `REAL`, `FLOOR`, `FLT`, `PACK`, and
`UNPK`; section 5 for REAL constant expressions; section 6.1 for the REAL
basic type; section 7 for REAL variables; sections 8.1, 8.2.2, and 8.2.4 for
REAL operands, arithmetic, and relations; section 9.1 for assignment; sections
9.2 and 10.1 for value and variable parameters and results; and section 10.2
for `ABS`, `FLOOR`, `FLT`, `PACK`, and `UNPK`.

Implementation starts from a reviewed Slice 8 with a green common gate. Any
Slice 8 review finding that affects scalar typing, `/`, constant folding, or
the QBE word path must be resolved before this slice changes those areas.

## Language and representation decisions

### REAL is IEEE 754 binary32

REAL occupies four bytes and uses the IEEE 754 binary32 representation. QBE
represents a REAL value with its `s` type. A REAL argument or result therefore
uses the platform ABI's single-precision floating-point class. A REAL variable
parameter remains an address and uses the QBE pointer class.

Project Oberon gives REAL a size of four bytes. Its code generator operates on
the binary32 sign, exponent, and fraction fields. QBE defines `s` as an IEEE
754 32-bit floating-point value and passes it according to the native C `float`
ABI. These choices agree without a compiler-specific calling convention.

OBNC can be configured to use `float`, `double`, or `long double`. That
portability choice is not needed here. This compiler has one target and chooses
the representation used by Project Oberon and directly supported by QBE.

The semantic constant representation is `f32`. The AST also stores a rounded
`f32` literal. The IR keeps REAL distinct from every word-sized type even
though REAL and INTEGER have the same storage size.

Module REAL variables use four-byte zero-filled data objects. Their initial
value is positive zero. Procedure locals remain uninitialized, as all current
procedure locals do. REAL value parameters are writable local copies under the
existing rule for basic types.

No target description or configurable floating-point layer is added. The
REAL size, QBE class, and runtime C type are direct constants in the same
places that already know the INTEGER and SET representations.

Declined: using IEEE binary64. It would make every INTEGER exactly
representable, but it would diverge from Project Oberon and double the storage
used by the language's single REAL type.

### Decimal literals round once to binary32

The lexer parses the complete token directly as an `f32`. This avoids parsing
to binary64 and then rounding a second time. The literal carried into the AST
is the exact value used by semantic analysis and generated code.

A literal that overflows to infinity receives a source diagnostic. A tiny
literal that underflows rounds to zero. Project Oberon also diagnoses an
exponent that is too large and rounds an exponent below its supported range to
zero.

The accepted syntax remains exactly the Report syntax. A decimal point is
required. Digits after the point and the scale factor are optional. The lexer
continues to distinguish `1..2` from a REAL literal beginning with `1.`.

QBE REAL immediates use the signed decimal spelling of the value's 32-bit bit
pattern in an `s` context. QBE constants are untyped bit strings, so this
preserves every binary32 value exactly. It also preserves negative zero and
any non-finite value produced by constant arithmetic.

Declined: emitting a host-formatted decimal REAL to QBE. A shortest decimal
would usually round back to the same value, but a bit-pattern immediate states
the representation directly and removes a second parser from the constant
path.

### Arithmetic rounds at binary32 precision

Unary plus accepts INTEGER or REAL and returns its operand unchanged. Unary
minus accepts INTEGER, REAL, or SET after this slice. REAL negation changes the
binary32 sign in the ordinary QBE `s` operation.

The operators `+`, `-`, and `*` accept two operands of the same numeric type.
INTEGER operands retain their current behavior. REAL operands produce a REAL
result through QBE single-precision arithmetic.

The source operator `/` accepts either two REAL operands or two SET operands.
It means REAL quotient in the first case and symmetric difference in the
second case. `DIV` and `MOD` remain INTEGER-only.

Equality, inequality, and all four ordering relations accept two REAL
operands. INTEGER and REAL never mix implicitly. This applies equally to
operators, assignment, arguments, results, and comparisons.

Constant REAL arithmetic uses `f32` operations at every source operator. It
does not compute a complete expression in binary64 and round only at the end.
This makes a folded expression follow the same precision as the QBE runtime
expression.

Ordinary REAL arithmetic follows IEEE 754 behavior. Overflow may produce an
infinity. Division by zero may produce an infinity or a NaN. No dynamic trap
is added for these ordinary arithmetic results because the Report does not
define one and QBE already supplies consistent binary32 behavior.

REAL relations follow IEEE 754 comparisons. A NaN is unequal to every value,
including itself. Every ordering comparison involving a NaN is false. Constant
folding uses the same rules.

The corpus pins the non-finite policy because it affects later library code and
cannot be inferred from the Report. The policy is part of this target's REAL
representation, not a new Oberon source feature. Source still has no spelling
for infinity or NaN.

### ABS is generic over the numeric types

`ABS(x)` accepts INTEGER or REAL and returns the argument's type. The existing
INTEGER overflow rule remains unchanged. REAL absolute value uses `fabsf`
through a small runtime wrapper.

The REAL form clears the sign of negative zero. It leaves a positive infinity
unchanged and returns a NaN for a NaN argument. Constant evaluation uses
`f32::abs` and therefore has the same behavior.

`ABS` remains one predefined identifier. Its checker handles the two accepted
types directly. No general generic-procedure signature system is introduced.

### FLT performs the explicit INTEGER-to-REAL conversion

`FLT(x)` requires one INTEGER expression and returns REAL. Runtime lowering
uses QBE `swtof` with an `s` result. Constant folding converts the `i32` value
directly to `f32`.

The conversion rounds to the nearest representable binary32 value. It does
not promise that every INTEGER is exact. In particular, the largest INTEGER
rounds to `2147483648.0` in this representation.

No assignment or arithmetic context inserts `FLT` automatically. The source
must request every INTEGER-to-REAL conversion explicitly.

### FLOOR checks that its result fits INTEGER

`FLOOR(x)` requires one REAL expression and returns the largest INTEGER not
greater than the argument. Negative inputs must round toward negative
infinity, as the examples in Report section 10.2 require.

The result exists only when the rounded binary32 argument is finite and lies
from `-2147483648.0` inclusive to `2147483648.0` exclusive. The lower endpoint
produces the minimum INTEGER value. The upper endpoint cannot produce an
INTEGER result.

If a constant argument is outside that domain, the compiler reports a source
diagnostic. If a dynamic argument is outside that domain, the program writes
`FLOOR result is outside INTEGER range` and a newline to standard error, then
exits with a nonzero status.

The dynamic operation calls a runtime wrapper. The wrapper checks finiteness
and range before calling `floorf` and converting the result to `int32_t`. This
avoids relying on QBE or C behavior for an out-of-range floating-point to
integer conversion.

The range applies after binary32 rounding. `FLT(2147483647)` is therefore
outside the `FLOOR` result domain because the REAL value has rounded upward.
This follows from the chosen representation rather than from the mathematical
INTEGER value passed to `FLT`.

Declined: letting QBE `stosi` decide an out-of-range result. QBE does not define
the language-level result needed here, and native targets need not agree.

### PACK and UNPK use the target math library

`PACK(x, n)` requires a writable REAL variable as its first argument. Its
second argument is an INTEGER expression. It replaces `x` with the binary32
result of multiplying `x` by two raised to `n`.

Runtime lowering calls a C wrapper around `ldexpf`. The wrapper receives the
address of `x` and the value of `n`. Overflow and underflow follow binary32 and
may produce an infinity, a subnormal value, or zero. No exponent-range trap is
added.

`UNPK(x, n)` requires a writable REAL variable and a writable INTEGER
variable. For a finite nonzero input, it preserves the sign of `x`, normalizes
the magnitude so it is at least one and less than two, and stores the
corresponding base-two exponent in `n`. Calling `PACK(x, n)` immediately after
`UNPK(x, n)` reconstructs the original finite value.

The Report states the positive normalization interval without explaining a
negative argument. OBNC preserves the sign and applies the interval to the
absolute value. This compiler chooses that conventional interpretation.

Project Oberon instead removes the sign from the normalized value and folds it
into an encoded exponent. Its `PACK` reverses that representation by editing
the binary32 exponent field. That pair round-trips, but its negative exponent
is not the mathematical exponent described for `PACK` in Report section 10.2.
This compiler does not adopt that target-specific encoding.

The Report also leaves zero unresolved because zero cannot satisfy the stated
normalization interval. This compiler defines `UNPK` of zero to store positive
zero in `x` and zero in `n`. The result round-trips through `PACK` and avoids an
arbitrary exponent.

Project Oberon exposes its minimum stored exponent for zero. OBNC's direct
`frexp` adaptation produces another exponent. The references therefore do not
provide one shared answer for this case.

`UNPK` of an infinity or NaN cannot produce a normalized REAL. It writes
`UNPK argument is not finite` and a newline to standard error, then exits with
a nonzero status.

The runtime implementation uses `frexpf` for a finite nonzero value. It doubles
the returned fraction and subtracts one from the returned exponent because C
normalizes the magnitude to one half inclusive and one exclusive.

The compiler resolves writable arguments from left to right. Each designator
is evaluated once. The `PACK` exponent expression is evaluated once after the
target address has been resolved.

Imported variables remain read-only. They cannot be passed as either modified
argument to `PACK` or `UNPK`.

## Constant evaluation and runtime lowering

### Folded and runtime values share one precision

REAL constants store `f32` rather than source text. Unary operations, binary
operations, relations, REAL `ABS`, `FLT`, and `FLOOR` all fold from that
representation.

The evaluator distinguishes INTEGER `/` from REAL `/` and SET `/` after type
checking has selected the source operation. It keeps the existing checked
INTEGER arithmetic and SET bit operations unchanged.

REAL overflow is not a constant-expression diagnostic. It produces the same
IEEE value as runtime arithmetic. The exceptions are a literal that cannot be
represented as a finite binary32 value and a constant `FLOOR` whose result
cannot fit INTEGER.

No constant optimization pass is added. Folding remains part of constant
declaration evaluation and the existing predefined-function path.

### The IR records the floating operand type

The IR gains a REAL type and a REAL immediate value. It also gains one explicit
INTEGER-to-REAL conversion operation for `FLT`.

The existing copy, unary, and binary instructions currently assume a QBE word
result. Give those instructions the source operand type where the emitter can
no longer infer it. Arithmetic and unary negation return that type. Relations
always return BOOLEAN but choose their QBE comparison from the operand type.

INTEGER, BOOLEAN, and SET still use QBE `w`. REAL uses QBE `s`. Integer-only
bit operations, shifts, `DIV`, and `MOD` remain word operations.

The runtime-backed predefined operations use ordinary IR calls. The calls name
the exact REAL or INTEGER value type for each value argument and result. A
modified argument uses the existing reference argument form.

### QBE emission uses the native single-precision ABI

REAL loads and stores use `loads` and `stores`. REAL arithmetic uses `add`,
`sub`, `mul`, `div`, and `neg` with an `s` result. REAL comparisons use the
single-precision comparison suffix and return `w`.

`FLT` emits `swtof` with an `s` result. No general conversion matrix is needed
because this slice adds only the conversion required by `FLT`. `FLOOR` remains
a checked runtime call.

REAL value parameters and results use `s` in function declarations and call
sites. REAL variable parameters use `l`. This agrees with the QBE ABI, which
passes `s` in the platform floating-point class and pointers in the integer
class.

## Modules and the calling convention

REAL works in every basic-type role available after Slice 8:

- A constant.
- A module variable.
- A procedure local.
- A value parameter.
- A variable parameter.
- A proper procedure argument.
- A function result.

A REAL value parameter is a writable local copy. Changing the formal does not
change the caller. A REAL variable parameter aliases the caller and can be
changed by assignment, `PACK`, or `UNPK`.

The module interface already carries basic semantic types and constant values.
Adding REAL to those enums carries exported REAL constants, variables,
parameter types, and results without a new interface format.

A cross-module gate exports every REAL form. The client folds an exported REAL
constant, reads an exported REAL variable, calls procedures with REAL value and
variable parameters, and receives a REAL function result.

The temporary native `Out` interface is not extended with REAL formatting.
Corpus programs observe exact, deliberately integral REAL results through
`FLOOR`. Formatted REAL output belongs to the portable library slice.

## What remains unsupported after this slice

- `TYPE` declarations, arrays, records, pointers, and procedure types remain
  in their later slices.
- CHAR, BYTE, strings, and character arrays remain unsupported until Slice 11.
- `CASE` remains limited to INTEGER until CHAR support arrives.
- `LEN`, `CHR`, `NEW`, and the CHAR form of `ORD` remain unavailable until
  their argument types exist.
- Selectors cannot yet be used as the modified arguments of `PACK` or `UNPK`
  because structured types have not arrived. Ordinary variables in every
  current storage class are supported.
- The portable library still consists of the temporary runtime-backed `Out`
  interface. It has no REAL output operation in this slice.
- The optional `SYSTEM` module remains outside the core roadmap.

The existing INTEGER overflow policy and SET domain policy remain unchanged.
REAL arithmetic uses the binary32 policy defined in this plan.

## Changes by file

### src/lexer.rs

Change the REAL token payload to `f32`. Parse a complete decimal literal
directly to that type. Diagnose a non-finite parsed literal as out of range and
retain underflow to zero.

Extend the lexer unit tests with the largest accepted scale, an overflowing
scale, an underflowing scale, an exponent with either sign, a missing exponent
digit, and the existing `1..2` disambiguation.

### src/ast.rs

Add a REAL expression containing the rounded `f32` value and its source
position. Extend expression position lookup for the new form.

No new source operator is needed. Slice 8 already retains unary plus and gives
the source `/` operator a distinct AST representation.

### src/parser.rs

Turn a REAL token into the new literal expression in `factor`. Remove REAL
from the unsupported literal branch.

No precedence rule changes. REAL uses the same numeric operator positions
already present in the Report grammar.

### src/sema.rs

Add `Type::Real` and `ConstValue::Real(f32)`. Add REAL to type display, IR
conversion, declarations, constants, procedure signatures, module interfaces,
and imported-interface reconstruction.

Install `REAL`, `FLOOR`, `FLT`, `PACK`, and `UNPK` in the universe scope.
Extend `ABS` to dispatch on INTEGER or REAL while preserving its result type.

Make unary, binary, and relation checking select behavior from the operand
types. Keep implicit INTEGER and REAL mixing illegal. Preserve Slice 8's SET
overloads and all existing INTEGER and BOOLEAN rules.

Extend constant evaluation with binary32 literals, arithmetic, relations,
REAL `ABS`, `FLT`, and checked `FLOOR`. Emit REAL operations with an explicit
IR operand type.

Lower REAL `ABS` and `FLOOR` as typed runtime calls. Lower `FLT` with the new IR
conversion. Lower `PACK` and `UNPK` through the existing writable-designator
path and typed runtime calls.

Keep generic predefined checking as direct cases for `ABS` and the existing
generic `ORD`. Do not introduce a generic signature framework for two small
families.

### src/ir.rs

Add a REAL type and REAL immediate value. REAL has a four-byte slot and the QBE
single-precision calling class.

Add operand type information to copy, unary, and binary instructions where
QBE emission needs it. Add an INTEGER-to-REAL conversion instruction for
`FLT`.

No REAL-specific address, load, store, parameter, call, or return instruction
is needed.

### src/qbe.rs

Map REAL storage, parameters, and results to QBE `s`. Emit a REAL immediate as
the signed decimal spelling of its binary32 bit pattern.

Emit typed REAL arithmetic and negation. Emit REAL relations with the `s`
comparison suffix and a `w` result. Emit the INTEGER-to-REAL conversion as
`swtof`.

Extend global-size, slot-size, load, store, argument, result, and return
matches for REAL. Preserve byte-for-byte QBE output for unchanged INTEGER,
BOOLEAN, and SET programs where the IR has only gained type annotations.

### runtime/oberon.c

Include the C math facilities and add four wrappers:

- `oberon_abs_real` calls `fabsf` and returns `float`.
- `oberon_floor` checks its domain, calls `floorf`, and returns `int32_t`.
- `oberon_pack` calls `ldexpf` and stores through a `float` pointer.
- `oberon_unpk` handles zero, rejects non-finite input, and otherwise adapts
  `frexpf` to the Report's normalization interval.

Add the two stable failure lines specified above. Keep source positions out of
these traps, consistently with the existing runtime failures.

### src/driver.rs

Link the generated executable with the platform math library after the object
and runtime inputs. No other driver or module-graph change is needed.

### agents/architecture.md

Extend the scalar-representation section added by Slice 8. Document the
binary32 representation, four-byte size, QBE `s` class, rounding precision,
and the checked result domain of `FLOOR`.

Document the runtime-backed `ABS`, `FLOOR`, `PACK`, and `UNPK` operations. Also
document the chosen zero and non-finite behavior of `UNPK`.

### src/main.rs

No change. REAL uses the existing build command and aggregate module pipeline.

## New corpus modules

`tests/corpus/` compiles each module, runs it, and compares standard output.

- `Reals.Mod` compares folded and runtime forms of unary plus, unary minus,
  addition, subtraction, multiplication, division, equality, inequality, and
  all four ordering relations. It observes integral results through `FLOOR`.
  It covers literals with empty fractional digits and positive and negative
  scale factors.
- `RealBuiltins.Mod` compares folded and runtime REAL `ABS`, `FLT`, and
  `FLOOR`. It covers `FLOOR(-1.5)`, negative zero, the minimum INTEGER value,
  and an INTEGER that does not convert exactly to binary32.
- `RealProcedures.Mod` exercises a module variable, locals, a writable value
  parameter, a variable parameter, a proper procedure argument, and a REAL
  function result. It proves that changing a value parameter does not change
  its actual and that changing a variable parameter does.
- `RealPacking.Mod` covers positive and negative PACK exponents. It covers
  PACK results that are infinite, subnormal, and zero. It covers positive and
  negative UNPK inputs, a subnormal input, and positive and negative zero. A
  reciprocal test proves that UNPK turns either zero into positive zero. An
  exponent function with a counter proves that PACK evaluates the expression
  once.
- `RealIeee.Mod` pins overflow, division by zero, NaN equality, NaN
  inequality, and NaN ordering for both folded and runtime expressions.
- `PredefinedShadow.Mod` is extended so module declarations can shadow
  `REAL`, `FLOOR`, `FLT`, `PACK`, and `UNPK`. Its existing predefined-name
  coverage remains.

`tests/corpus/modules/real-api/` supplies the required cross-module coverage.

- `RealSupport.Mod` exports a REAL constant, a REAL variable, a procedure with
  value and variable REAL parameters, and a function returning REAL. Its
  module body initializes the exported variable.
- `RealApi.Mod` imports the support module. It folds the exported constant,
  reads the variable, calls every exported procedure form, and converts exact
  results to INTEGER for output.

`tests/errors/` must fail with exact diagnostics.

- `RealBad.Mod` covers implicit INTEGER and REAL mixing in arithmetic,
  relations, assignment, arguments, and results. It also covers `/` on
  INTEGER, `DIV` and `MOD` on REAL, Boolean operators on REAL, and REAL and SET
  mixing.
- `RealBuiltinBad.Mod` covers wrong argument types and arities for REAL `ABS`,
  `FLT`, `FLOOR`, `PACK`, and `UNPK`. It requires variable arguments where the
  Report modifies them and rejects either proper procedure in an expression.
- `RealLiteralRange.Mod` contains a decimal literal that overflows binary32.
- `FloorConstRange.Mod` covers constant values below and above the INTEGER
  result domain. One case reaches the upper boundary through `FLT` rounding.
  Separate cases pass constant infinity and constant NaN, which proves that
  the evaluator checks finiteness rather than relying only on comparisons.

`tests/errors/modules/real-import-write/` proves that imported variables stay
read-only through the new modifying procedures.

- `RealImportWrite.Mod` tries ordinary assignment, a REAL variable parameter,
  `PACK`, and both modified positions of `UNPK` where their types permit.
- `RealImportSupport.Mod` exports the REAL and INTEGER variables and procedure
  required by the negative root.

`tests/failures/` compiles each module, runs it, and compares stable standard
error after a nonzero exit.

- `FloorRangeLow.Mod` passes a dynamic REAL below the minimum INTEGER value to
  `FLOOR`.
- `FloorRangeHigh.Mod` passes the dynamic upper endpoint to `FLOOR`.
- `FloorInfinity.Mod` produces a dynamic infinity before calling `FLOOR`.
- `FloorNan.Mod` produces a dynamic NaN before calling `FLOOR`.
- `UnpkInfinity.Mod` produces a dynamic infinity before calling `UNPK`.
- `UnpkNan.Mod` produces a dynamic NaN before calling `UNPK`.

The four FLOOR failures expect the same range line. Both UNPK failures expect
their separate non-finite line.

## Verification

1. Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
   `cargo test`, and `git diff --check`.
2. Run every new positive binary directly. Confirm zero exit status, empty
   standard error, and byte-for-byte expected standard output.
3. Read the folded and runtime columns in `Reals.Mod`, `RealBuiltins.Mod`, and
   `RealIeee.Mod`. Confirm every intended pair agrees at binary32 precision.
4. Inspect `build/Reals.ssa`. Confirm REAL arithmetic uses `s`, REAL relations
   use the single-precision comparison suffix, and REAL immediates preserve
   their exact 32-bit patterns.
5. Inspect `build/RealProcedures.ssa`. Confirm REAL values use `s`, REAL
   variable parameters use `l`, REAL locals allocate four bytes, and REAL
   results return `s`.
6. Inspect `build/RealBuiltins.ssa`. Confirm `FLT` uses `swtof`. Confirm REAL
   `ABS` and `FLOOR` call wrappers with `s` arguments and the correct result
   classes.
7. Inspect `build/RealPacking.ssa`. Confirm PACK passes one REAL address and
   one INTEGER value. Confirm UNPK passes both addresses. Confirm the exponent
   function is called once.
8. Inspect the cross-module IL. Confirm the exported REAL global has one
   four-byte data object and every module uses the same symbol and `s` loads.
9. Run the FLOOR and UNPK failure binaries directly. Confirm a nonzero status
   and the exact stable line for each failure.
10. Rebuild representative INTEGER, BOOLEAN, and SET modules and compare their
    IL with the pre-slice result. The typed IR extension must not change their
    behavior or calling classes.
11. Confirm no newly valid REAL program can reach an unsupported diagnostic,
    panic, QBE error, or mismatched C ABI call.

## Order of work

1. Start from the reviewed Slice 8 baseline. Capture representative word-class
   and SET IL for the regression comparison.
2. Change REAL token precision, add literal range handling, add the AST form,
   and let the parser accept REAL factors.
3. Add REAL to semantic types, constants, the universe scope, declarations,
   procedure signatures, and module interfaces.
4. Add the REAL IR type, exact immediate, typed operations, conversion, and QBE
   mappings. Establish four-byte storage and the `s` calling class before
   lowering source arithmetic.
5. Implement unary operations, arithmetic, relations, and constant folding
   together. Add folded and runtime comparison tests at this point.
6. Add generic REAL `ABS`, `FLT`, and checked `FLOOR`. Add the runtime math
   wrappers and math-library link at the same time.
7. Add `PACK` and `UNPK` through the writable-designator path. Pin sign, zero,
   non-finite, evaluation-order, and imported-read-only behavior.
8. Add all positive, negative, runtime-failure, and cross-module gates. Add a
   regression module for every bug found during implementation.
9. Update the architecture document and complete the verification list.
