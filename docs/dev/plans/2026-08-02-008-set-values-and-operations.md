# Slice: SET values and operations

## Context

After Slice 7 the compiler builds a graph of source modules. It supports
exported INTEGER and BOOLEAN constants, variables, and procedures. Those values
can cross module boundaries through the in-memory interface and the existing
word-sized calling convention.

The lexer already recognizes `SET`, `IN`, braces, and `..`. The parser still
rejects `IN`, set constructors, and `/`. Semantic analysis has no SET type or
value. `INCL` and `EXCL` are absent from the universe scope. `ORD` accepts only
BOOLEAN.

This slice adds the complete SET basic type. It adds set constructors, set
ranges, every set operator, membership, `INCL`, `EXCL`, and the SET form of
`ORD`. SET values also work in every scalar storage and procedure role that the
compiler already supports.

Report sections implemented: section 4 for the predefined identifiers `SET`,
`INCL`, and `EXCL`; section 5 for SET constant expressions; section 6.1 for the
SET basic type; sections 8.1 and 8.2 for set constructors and operators; section
9.1 for SET assignment; sections 9.2 and 10.1 for SET value and variable
parameters; and section 10.2 for `INCL`, `EXCL`, and `ORD`.

Slice 7 has one open review finding about accepting a root source with the wrong
file extension. That finding is unrelated to SET. It should be closed before
Slice 8 implementation starts so this slice begins from a reviewed, green
baseline.

## Language and representation decisions

### SET contains the integers from 0 through 31

Report section 6.1 leaves the upper SET element implementation-dependent. This
compiler chooses 31. A SET can therefore contain exactly the INTEGER values
from 0 through 31, inclusive.

A SET value is one 32-bit bit vector. Bit `n` records membership of element
`n`. The empty set is zero. The full set is `0FFFFFFFFH` as a bit pattern.

The semantic constant representation is `u32`. This keeps set arithmetic free
from signed overflow and makes complement cover exactly the 32 supported bits.
The IR still gives SET its own type and value variants. QBE stores the value in
one `w`, just as it stores INTEGER and BOOLEAN, but the semantic types never
become interchangeable.

Module variables use the existing zero-filled static storage. They therefore
begin as the empty set. Procedure locals remain uninitialized, like the
existing scalar locals, and correct programs must assign them before reading
them. SET value parameters and results use the QBE word class. SET variable
parameters use the existing address class.

The set limit is a plain constant next to the semantic SET implementation. No
target description or configurable word-size layer is added. This compiler has
one target, and the representation is now part of that target's documented
behavior.

Project Oberon also uses a 32-bit SET and assigns it four bytes. OBNC uses an
unsigned integer with the same width as its INTEGER configuration. The fixed
32-bit choice matches this compiler's existing 32-bit INTEGER and QBE word
operations.

### ORD exposes the SET bit pattern as an INTEGER

Report section 10.2 says that `ORD` returns the ordinal number of a SET. It does
not define how the full 32-bit SET range maps into a signed INTEGER.

This compiler reinterprets the SET bit pattern as a signed 32-bit INTEGER.
`ORD({0})` is 1. `ORD({31})` is `MIN(INTEGER)`. `ORD(-{})` is -1.

This matches Project Oberon's no-op lowering for `ORD`. It also keeps the
constant and runtime forms identical. Runtime `ORD` emits no instruction. The
constant evaluator casts the `u32` bit pattern to `i32`.

Declined: limiting SET to 31 elements so every `ORD` result is nonnegative. The
Report makes the SET limit implementation-dependent, but the roadmap already
selects a natural 32-bit representation. Giving up the high bit would make the
representation less direct without clarifying the Report's use of “ordinal
number.”

### Operators are selected by operand type

Report section 8.2 overloads `+`, `-`, `*`, and `/`. For SET operands they mean
union, difference, intersection, and symmetric difference. Unary `-` on SET
means complement. Equality and inequality are the only ordinary relations
defined for SET.

The AST continues to represent source operators rather than resolved
operations. Semantic analysis selects INTEGER or SET behavior after it knows
both operand types. The `/` token gets its own AST operator because INTEGER
`DIV` is a different token and Slice 9 will later give `/` its REAL meaning.

Unary `+` needs to remain in the AST instead of being discarded by the parser.
Report section 8.2.2 defines unary `+` only for numeric operands. It is not a
SET identity operator. Retaining it also fixes the existing acceptance of
`+TRUE`.

Unary `-` accepts INTEGER and SET in this slice. Slice 9 extends it to REAL.
Unary `~` remains BOOLEAN negation only.

Project Oberon extends `LSL`, `ASR`, and `ROR` to SET. The Report lists INTEGER
arguments for those procedures. This compiler does not adopt that extension.

### Every named set element uses one range rule

An INTEGER used as a set element must lie between 0 and 31. This rule applies
to all of these positions:

- A single element in a set constructor.
- Either endpoint of a set range.
- The left operand of `IN`.
- The second argument of `INCL`.
- The second argument of `EXCL`.

If the compiler can evaluate the INTEGER as a constant expression, an
out-of-range value is a source diagnostic. This includes a literal, a named
constant, and a compound constant expression. An out-of-range value must not
be deferred to runtime merely because it was written as `16 + 16`.

If the INTEGER is dynamic, the generated program checks both bounds before it
performs a shift. A failure calls one runtime trap and exits. QBE reduces shift
counts modulo the word width, so an unchecked value of 32 would otherwise act
like zero.

The Report requires only an INTEGER left operand for `IN`. It does not state
what happens outside the implementation's SET domain. Applying the same range
rule to membership makes the result independent of the backend and agrees with
the roadmap's requirement for checked dynamic elements.

Declined: reducing an element modulo 32. That would make `{32}` equal `{0}`.
It would also make `32 IN {0}` true.

Declined: returning FALSE for out-of-range membership while trapping for the
same value in a constructor. A single checked domain is smaller and easier to
explain.

### Reversed ranges are empty after their endpoints are checked

Report section 8.2 states that `{m .. n}` is empty when `m` is greater than
`n`. This rule applies to constant and dynamic ranges.

Both endpoint expressions are evaluated exactly once, from left to right.
Both endpoints are range-checked before the range result is selected. A
reversed range does not hide an invalid endpoint.

For an in-range dynamic range, lowering compares the endpoints. The reversed
branch produces zero. The other branch builds an inclusive mask without ever
shifting by 32.

One suitable mask is the intersection of two values. The first has every bit
at or above the low endpoint set. The second has every bit at or below the high
endpoint set. Its shift counts are `low` and `31 - high`, so both stay within
the checked range from 0 through 31.

Every element and range in a constructor is evaluated once in source order.
Their masks are combined with union. A function call used as an endpoint must
therefore run once even when the range is reversed.

## Constant evaluation and runtime lowering

### Constructors and operators share the same bit rules

The constant evaluator and runtime lowering use these operations:

- Union uses bitwise OR.
- Difference clears the right-hand bits from the left-hand value.
- Intersection uses bitwise AND.
- Symmetric difference uses bitwise XOR.
- Complement uses XOR with the full-set mask.
- Equality and inequality compare the complete bit pattern.
- Membership tests the checked element's bit.

Difference can lower as an intersection with the complemented right operand.
No subtraction is involved. Complement cannot expose bits outside the SET
domain because all 32 bits belong to the domain.

The IR gains bitwise XOR. Its existing bitwise AND, bitwise OR, shifts,
comparisons, branches, and copies are enough for the other operations. No
special SET instruction is needed.

Constant folding covers constructors, ranges, every set operator, equality,
inequality, membership, and `ORD`. The folding code must use the same range
helper and bit formulas as runtime lowering. A constant range checks both
endpoints before applying the reversed-range rule.

The semantic checker must distinguish an expression that is not constant from
a constant expression that is invalid. Set lowering needs that distinction so
it can diagnose a known bad element while still emitting a runtime check for a
variable or function result. A small optional constant-evaluation path is
enough. It must not add a general optimization pass.

### Dynamic checks precede every shift

The existing shift-count check from Slice 6 provides the control-flow shape.
SET lowering should use one analogous helper for values from 0 through 31.
Known good constants need no generated check. Known bad constants produce a
diagnostic and no shift.

For a dynamic value, the helper compares against zero and 31. It branches to
`oberon_set_element_range` on failure. The successful block performs the
shift.

The runtime trap writes `SET element out of range` and a newline to standard
error. It exits with a nonzero status. It carries no source position, which is
consistent with the existing arithmetic and control-flow traps.

The constructor evaluates an endpoint before checking it. `IN` evaluates its
left operand and then its right operand, in the compiler's existing
left-to-right expression order. `INCL` and `EXCL` resolve their writable target
before evaluating the element argument, as other variable-argument predefined
procedures do.

### INCL and EXCL are writable operations

`INCL(v, x)` requires a writable SET variable as its first argument and an
INTEGER as its second. After the element check, it stores the union of `v` and
the singleton set `{x}`.

`EXCL(v, x)` has the same argument rules. It stores the difference between `v`
and `{x}`.

The existing writable-designator path covers module variables, locals, basic
value parameters, and SET variable parameters. It also rejects constants,
expressions, and imported variables. An imported exported SET variable remains
readable but cannot be changed through `INCL`, `EXCL`, or an ordinary variable
parameter.

Both names are proper predefined procedures. They cannot occur in an
expression or a constant declaration. Their arity is exactly two.

`ORD` becomes the first predefined function with two accepted argument types.
Its checker accepts BOOLEAN or SET and returns INTEGER. This should be handled
as a small special case rather than introducing a general generic-procedure
signature framework.

## Modules and the calling convention

SET must work in every basic-type role available after Slice 7:

- A constant.
- A module variable.
- A procedure local.
- A value parameter.
- A variable parameter.
- A proper procedure argument.
- A function result.

A SET value parameter is a writable local copy, following the existing rule
for basic value parameters. Changing it with `INCL` or `EXCL` does not change
the caller. A SET variable parameter aliases the caller and can be changed.

The module interface already stores semantic types and constant values. Adding
SET to those enums is enough to carry exported SET constants, variables,
parameter types, and results. No new interface member kind is needed.

A cross-module gate must export all of these forms. It must read an exported
SET variable, call procedures with SET values, call a function returning SET,
and fold an exported SET constant in the client. A negative cross-module gate
must try to change an imported SET variable with both `INCL` and `EXCL`.

## What remains unsupported after this slice

- REAL remains unsupported until Slice 9. The `/` operator therefore accepts
  SET operands only in this slice.
- `TYPE` declarations, arrays, records, pointers, and procedure types remain in
  their later slices.
- CHAR and BYTE remain unsupported. The CHAR form of `ORD` arrives with CHAR in
  Slice 11.
- `CASE` remains limited to INTEGER. The Report does not define SET as a case
  expression type.
- `LSL`, `ASR`, and `ROR` remain INTEGER-only, as required by Report section
  10.2.
- The portable library still consists of the temporary runtime-backed `Out`
  interface.
- The optional `SYSTEM` module remains outside the core roadmap.

The known INTEGER overflow boundaries from earlier slices remain unchanged.
SET operations are fixed-width bit operations and introduce no arithmetic
overflow policy.

## Changes by file

### src/ast.rs

Add a SET expression containing a list of elements. Each element stores a low
expression and an optional high expression. Keep the source positions on the
endpoint expressions so diagnostics point at the value that is out of range.

Add a source `/` binary operator and an `IN` binary operator. Add an explicit
unary plus operator. Keep unary minus as one source operator because semantic
analysis chooses INTEGER negation or SET complement.

### src/lexer.rs

No change. Every token needed by this slice already exists.

### src/parser.rs

Parse set constructors in `factor`, including the empty form, comma-separated
elements, and optional ranges. Each endpoint uses the full `expression`
production required by the Report grammar.

Parse `IN` as a relation. Continue to reject `IS` until pointer and record type
tests arrive.

Parse `/` as its own multiplication-level source operator. Stop reporting it
as unsupported real division. Semantic analysis diagnoses non-SET uses until
REAL arrives.

Retain unary plus in the AST instead of dropping it. This lets semantic
analysis reject SET and BOOLEAN operands.

### src/sema.rs

Add `Type::Set` and `ConstValue::Set(u32)`. Add SET to type display, IR
conversion, variable declarations, constants, procedure signatures, exported
interfaces, and imported interface reconstruction.

Install `SET`, `INCL`, and `EXCL` in the universe scope. Extend `ORD` to accept
SET without changing its BOOLEAN behavior.

Add SET constructor type checking, optional constant evaluation, and lowering.
Add the shared element range rule. Apply it to constructors, membership,
`INCL`, and `EXCL`.

Make unary and binary checking dispatch on operand types. INTEGER behavior must
stay unchanged. SET gets the Report's four binary operations, complement,
equality, inequality, and membership. Ordering, Boolean operators, numeric
`DIV`, and `MOD` remain invalid on SET.

Extend constant evaluation with SET values and operations. Keep the static and
runtime range rules side by side so a later change cannot update one without
the other.

Add lowering for `INCL` and `EXCL` through the existing writable-designator
path. Keep them out of the function-like predefined signature path.

### src/ir.rs

Add a SET type and SET immediate value. SET uses a four-byte slot and the word
calling class. Add bitwise XOR to the binary operations.

No SET-specific address, load, store, parameter, call, or return instruction is
needed.

### src/qbe.rs

Map SET storage and parameters to QBE `w`. Emit a SET immediate as the signed
decimal spelling of its 32-bit bit pattern. This keeps values with bit 31 set
valid in QBE while preserving all bits.

Emit bitwise XOR as QBE `xor`. Extend the exhaustive storage-size and class
matches for SET.

### runtime/oberon.c

Add `oberon_set_element_range` in the same shape as the existing traps. It
writes the stable error line and exits.

### docs/dev/architecture.md

Add a short scalar-representation section. Document the SET domain, bit-vector
layout, four-byte size, QBE word class, and `ORD` reinterpretation. Document
that element-producing and membership operations check the domain before a
shift.

### src/driver.rs and src/main.rs

No change. SET uses the existing aggregate IR, source graph, QBE invocation,
and runtime link.

## New corpus modules

`tests/corpus/` compiles each module, runs it, and compares standard output.

- `Sets.Mod` compares folded and runtime values for union, difference,
  intersection, symmetric difference, complement, equality, inequality, and
  membership. It covers the empty set, full set, elements 0 and 31, an
  inclusive range, and a reversed range. It prints `ORD` results for the empty
  set, bit 31, and the full set.
- `SetRanges.Mod` uses variable endpoints for ascending and reversed ranges.
  Endpoint functions with counters prove left-to-right, exactly-once
  evaluation. A reversed range still evaluates both calls.
- `SetProcedures.Mod` exercises module variables, locals, value parameters,
  variable parameters, proper procedure arguments, and function results. A
  value parameter is changed without changing its actual. A variable
  parameter is changed with both `INCL` and `EXCL`.
- `PredefinedShadow.Mod` is extended so a module declaration can shadow the
  newly predefined SET names. The existing INTEGER and BOOLEAN checks remain.

`tests/corpus/modules/set-api/` supplies the required cross-module coverage.

- `SetSupport.Mod` exports a SET constant, a SET variable, a procedure with
  value and variable SET parameters, and a function returning SET. Its module
  body initializes the exported variable.
- `SetApi.Mod` imports the support module. It folds the exported constant,
  reads the exported variable, calls every exported procedure form, and prints
  the results.

`tests/errors/` must fail with exact diagnostics.

- `SetBad.Mod` covers INTEGER and SET operand mixing, `/` on INTEGER before
  REAL support, ordering SET values, BOOLEAN elements, `DIV` and `MOD` on SET,
  `~` on SET, unary `+` on SET, and SET assignment mismatches.
- `SetElementConst.Mod` covers -1 and 32 as constructor elements. It also
  covers bad low and high range endpoints. At least one value is a named
  constant and one is a compound constant expression.
- `SetBuiltinBad.Mod` covers the wrong first argument to `INCL` and `EXCL`, a
  non-variable first argument, a BOOLEAN element, wrong arity, use of either
  proper procedure as a value, and `ORD` on INTEGER.
- `SetInConstBad.Mod` covers constant out-of-range left operands of `IN` at
  both ends of the domain.

`tests/errors/modules/set-import-write/` proves that imported SET variables are
read-only.

- `SetImportWrite.Mod` calls both `INCL` and `EXCL` on an imported variable. It
  also supplies that variable to a SET variable parameter.
- `SetImportSupport.Mod` exports the variable and the procedure used by the
  negative root.

`tests/failures/` compiles each module, runs it, and compares stable standard
error after a nonzero exit. Separate roots are needed because the first trap
ends the process.

- `SetElementLow.Mod` and `SetElementHigh.Mod` construct singleton sets from
  variables holding -1 and 32.
- `SetRangeLow.Mod` and `SetRangeHigh.Mod` use an invalid dynamic endpoint in a
  range. One case is reversed, proving that endpoint validation happens before
  the empty-range result is selected.
- `SetInRange.Mod` uses an out-of-range variable as the left operand of `IN`.
- `InclRange.Mod` and `ExclRange.Mod` use an out-of-range variable as the
  element argument.

Every failure module expects the same `SET element out of range` line. This
pins the shared runtime rule rather than backend shift behavior.

## Verification

1. Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
   `cargo test`, and `git diff --check`.
2. Run every new positive binary directly. Confirm zero exit status, empty
   standard error, and byte-for-byte expected standard output.
3. Read the folded and runtime columns in `Sets.Mod`. Confirm every pair is
   equal, including bit 31 and the full-set complement.
4. Inspect `build/Sets.ssa`. Confirm union uses `or`, intersection uses `and`,
   symmetric difference uses `xor`, and difference clears the right-hand bits.
5. Inspect `build/SetRanges.ssa`. Confirm each endpoint call appears once.
   Confirm the range comparison precedes the two safe mask shifts.
6. Inspect one dynamic constructor, `IN`, `INCL`, and `EXCL`. Confirm each
   range guard precedes its shift and branches to
   `$oberon_set_element_range`.
7. Inspect `build/SetProcedures.ssa`. Confirm SET values use `w`, SET variable
   parameters use `l`, SET locals allocate four bytes, and SET results return
   `w`.
8. Inspect the cross-module IL. Confirm the exported SET global has one
   four-byte data object and both modules use the same symbol.
9. Rebuild an unchanged scalar module and compare its IL with the pre-slice
   result. Adding SET variants must not change INTEGER or BOOLEAN output.
10. Confirm no newly valid SET program can reach an unsupported diagnostic,
    panic, or QBE error.

## Order of work

1. Close the unrelated Slice 7 filename finding and establish the common green
   baseline.
2. Add the AST forms and parser support for constructors, `/`, `IN`, and unary
   plus. Add parser-focused coverage before semantic lowering.
3. Add SET to semantic types, constants, the universe scope, and module
   interfaces.
4. Add the SET IR type, SET immediates, bitwise XOR, and QBE mappings.
5. Implement constructor checking, constant folding, and runtime lowering
   together. Add the shared element-domain check at this point.
6. Extend unary operations, binary operations, equality, and membership.
7. Add `INCL`, `EXCL`, and the SET form of `ORD`.
8. Add the runtime trap and all positive, negative, runtime-failure, and
   cross-module gates. Add a regression module for every bug found during the
   work.
9. Update the architecture document and complete the verification list.
