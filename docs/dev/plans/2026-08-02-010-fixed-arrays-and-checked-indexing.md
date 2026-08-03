# Slice: Fixed arrays and checked indexing

## Context

After Slice 9 the compiler supports all of the Report's scalar types except
CHAR and BYTE. INTEGER, BOOLEAN, SET, and REAL work in constants, module
variables, procedure locals, scalar parameters and results, and module
interfaces. Semantic analysis lowers scalar values and addresses to typed IR;
QBE emission does not resolve names or choose source-language operations.

The parser still rejects every `TYPE` section. A variable's source type is a
qualified identifier rather than a general type expression, and the only
selector retained in the AST is a period. Semantic types are small copyable
values, module interfaces do not export type names, and IR storage always has
the size and alignment of one scalar. There is consequently no place yet to
record aggregate identity, layout, an indexed address, or a whole-value copy.

This slice adds type declarations, named and inline fixed arrays,
multidimensional arrays, index selectors, whole-array assignment, and `LEN`
for fixed arrays. It gives arrays module and procedure-local storage and makes
an exported named array type usable by clients. Every executed index is
checked before its element address is formed.

Report sections implemented: section 4 for type declarations, textual scope,
export marks, and qualified type names; section 5 for constant array lengths;
section 6 and section 6.2 for named, inline, fixed, and multidimensional array
types; section 7 for array variables; section 8.1 for index selectors and the
`A[E1, ..., En]` abbreviation; section 9.1 for element and whole-array
assignment; section 9.2 for evaluating a selected scalar variable passed as a
`VAR` actual; section 10 for local type declarations; section 10.2 for `LEN`;
and section 11 for exported and imported type names.

Implementation starts from a reviewed Slice 9 with a green common gate. Slice
9 changes the scalar type and IR matches for REAL, so its representation,
typed copy instructions, and QBE `s` path are the baseline rather than code to
rework around.

## Language and representation decisions

### An ARRAY constructor creates one type identity

Semantic types stop being copyable values. A basic type remains a small enum
case. An array type is a shared descriptor containing its length, element type,
size, and alignment. Equality of array types compares descriptor identity, not
their printed shape.

Each source `ARRAY` constructor creates one descriptor. All names in one
variable declaration share the type resolved for that declaration, so these
variables have identical types:

```oberon
VAR a, b: ARRAY 8 OF INTEGER;
```

Two separately written constructors create different types even when their
lengths and element types match. A type declaration whose right side is an
existing qualified type is an alias and retains that existing identity:

```oberon
TYPE
  Row = ARRAY 8 OF INTEGER;
  OtherRow = ARRAY 8 OF INTEGER;
  Alias = Row;
```

`Row` and `Alias` are identical. `OtherRow` is not identical to either one.
This applies to nested arrays as a whole, not just their outermost dimension.
It follows the Report's requirement for the same type and OBNC's identity
rule. Project Oberon's additional shallow compatibility check for equal array
length and base descriptors is not adopted; it would make separately declared
one-dimensional arrays compatible while leaving equivalent multidimensional
declarations incompatible.

Use shared owned descriptors, such as `Rc`, rather than allocating compiler-
global numeric type identifiers. Cloning a semantic type for a symbol or an
interface preserves descriptor identity. This is enough for the current
in-memory module graph and stays easy to replace when recursive record and
pointer types require a richer representation.

Declined: structural equivalence based only on lengths and element shapes. It
would erase the identity that Slice 10 is required to preserve across module
boundaries.

### Type declarations accept aliases and array expressions

The source type AST has two forms in this slice:

- A qualified type name.
- `ARRAY` with one or more length expressions and a source element type.

This supports a type alias, a named array, an inline array in a variable
declaration, and recursively nested inline arrays. RECORD, POINTER, and
PROCEDURE type expressions continue to receive explicit unsupported
diagnostics for their later slices.

Type declarations are analyzed in textual order. The right side is resolved
before its identifier is declared, so it can use earlier types and constants
but not itself or a later type. The pointer-specific forward reference is not
introduced early. A local type is visible to nested procedures under the
existing enclosing-type lookup rule. An export mark remains legal only at
module scope.

The parser represents a comma list in one array constructor directly, while
semantic analysis expands it from right to left. Thus
`ARRAY N0, N1 OF T` has exactly the same semantic type shape as
`ARRAY N0 OF ARRAY N1 OF T`, as required by Report 6.2.

### Lengths are non-negative constant INTEGER values

Each length expression is checked with the existing constant evaluator. It
must produce an INTEGER and must be at least zero. A variable, a user procedure
call, a REAL or BOOLEAN value, a negative value, and an invalid constant
operation all receive source diagnostics. Zero is an ordinary valid length.

Array layout uses checked arithmetic and a hard target aggregate-storage limit
of one GiB. This is deliberately below QBE's signed stack-offset limit and the
current platform's small-code-model data reach, leaving comparable headroom
for generated code, runtime objects, and linker placement. An array whose
element-size multiplication overflows, or whose final size exceeds that
limit, receives an `array type exceeds target object-size limit` diagnostic.
The complete aligned storage of each procedure and source module is checked
against the same limit, so several individually valid locals or globals cannot
create a QBE frame or data unit outside it. The driver also rejects a build
whose modules together exceed the static-data limit before invoking QBE.

This is an implementation resource limit, not an Oberon type rule. Keep it as
one direct constant shared by semantic layout and final program validation.
There is no configurable target-layout object for the one current target.

Declined: silently replacing a bad length with one so that analysis can
continue. A placeholder may be kept internally after recording a diagnostic,
but no symbol carrying that placeholder may make an invalid module
successfully compile.

### Arrays are contiguous and have their element alignment

An array stores exactly `length * element_size` bytes, with element `i` at
`base + i * element_size`. Its alignment is its element type's alignment. All
types available through Slice 10 have size and alignment four, so every
nonempty scalar array and every nested array is four-byte aligned. Keeping the
rule on the type rather than hard-coding four prepares the same layout
calculation for CHAR, BYTE, and records without introducing a general target
description.

A zero-length array has size zero and retains its element alignment. It gains
no hidden element and no minimum payload byte. QBE accepts zero-byte data and
zero-byte `alloc4` storage, and no valid selector can dereference that storage.
This choice is required for later records containing zero-length arrays.

Module arrays are emitted as zero-filled data objects of their complete byte
size. Procedure-local arrays reserve the same complete size in the procedure's
activation record and, like existing scalar locals, remain uninitialized.

### Indexing is a checked address operation

The AST retains every bracket selector and every expression in its comma list.
Semantic analysis treats `a[i, j]` as `a[i][j]`. At each dimension the current
designator must have array type and the index must have type INTEGER. The
selector's resulting type is that array's element type.

Index expressions are evaluated from left to right and exactly once. For each
dimension, its value is checked against zero inclusive and that dimension's
length exclusive. The check completes before the value is widened, scaled, or
added to the base address, and before the next dimension is evaluated. This
makes a zero-length array fail every dynamic index without relying on pointer
arithmetic or a later load.

A constant index outside its domain is a source diagnostic. A valid constant
index in an ordinary executable expression still passes through the
checked-index IR operation. A required constant context instead validates the
selector statically and emits no executable operation. The compiler has no
optimization pass, and keeping one executable lowering path makes the IR
invariant literal: every executed index carries its length and checks it before
address calculation.

A dynamic failure writes this exact line to standard error and exits with a
nonzero status:

```text
array index out of bounds
```

The diagnostic does not include a source position yet, consistently with the
existing DIV, SET, ASSERT, FLOOR, and UNPK runtime failures.

The designator keeps the read-only state of its base variable. An imported
array may be indexed and read, but neither the array nor an element or nested
subarray may be assigned. An element of a writable array is an ordinary
writable designator: scalar assignment, a scalar `VAR` actual, `INC`, `DEC`,
`INCL`, `EXCL`, `PACK`, and `UNPK` all use the same resolved address and do not
evaluate its selectors again.

### Whole-array assignment is a value copy

An array assignment requires a writable array designator on the left and an
array designator of the identical semantic type on the right. It copies the
complete byte representation. It does not make the destination refer to the
source. Mutating either array after the assignment therefore cannot change the
other one.

Resolve the destination designator once, then the source designator once, and
only then copy. This preserves source evaluation order and also makes selected
subarray assignment correct. Identically typed fixed array regions are either
disjoint or fully overlapping, but the runtime copy uses `memmove` so even the
self-assignment case has an explicit safe meaning.

A zero-length assignment still resolves both designators and executes every
selector and bounds check. Its byte copy then has count zero. This matters for
side effects and for a selected zero-length subarray.

Arrays are not silently turned into pointer values. An array designator is
accepted as a whole value only in whole-array assignment and the procedure
contexts implemented by later slices. Arithmetic, ordinary relations,
conditions, constants, scalar arguments, and function results reject it.

### LEN returns the selected fixed dimension

Install `LEN` in the universe scope as a predefined function. In this slice it
requires exactly one designator denoting a fixed array variable and returns
INTEGER. `LEN(a)` is the outer length. `LEN(a[i])` is the length of the selected
inner array.

The argument designator is still evaluated. In particular, the index in
`LEN(a[f()])` is evaluated once and checked before the statically known inner
length is returned. This agrees with Project Oberon's selector lowering and
prevents `LEN` from erasing source side effects or an invalid selection.

Runtime lowering always resolves the designator first and then returns the
descriptor length as an INTEGER immediate. It does this even when every
selector expression is constant, so ordinary `LEN(a[0])` retains the selected
designator's check.

In a required constant context, such as a constant declaration or another
array's length, constant evaluation may fold `LEN` when every selector is
constant and statically in range. The array variable itself need not be a
constant because a fixed length is a type property. A dynamic selector makes
the call nonconstant even though its eventual result is statically known. An
out-of-range constant selector is a source diagnostic. No executable check or
selector effect is erased because required constant expressions cannot contain
those effects.

No general generic-predefined signature framework is added for this one
structured argument.

`LEN` of open arrays and strings arrives with those types in later slices.

## Semantic types and module interfaces

### Scalar value types and storage types become distinct questions

Semantic `Type` gains the shared array descriptor and small helpers for:

- Type identity.
- Scalar IR type, when one exists.
- Byte size and alignment.
- Array length and element type.
- A printable diagnostic name or shape.

Code that loads, stores, passes, or returns a value must ask for a scalar IR
type. Code that declares storage asks for layout. This prevents an array from
reaching QBE through an accidental scalar `loadw`, `loads`, or call argument.

The refactor removes `Copy` assumptions from semantic types but does not add a
general type arena, visitor, or compatibility framework. Direct helper methods
and matches are enough for basic types plus one aggregate form.

### Interfaces carry exported types without serializing them

Add a type member to the in-memory module interface. An exported type name is
present as its semantic type; a private type name is absent. A client resolves
`M.T` through the same module member lookup used for constants, variables, and
procedures.

Cloning an interface clones shared type handles, so a client declaration using
`M.T`, an exported variable whose type is `T`, and another module that imports
the same `T` all retain one identity. A re-exported alias also retains the
original identity. No symbol file, interface cache, structural reconstruction,
or module-qualified numeric type registry is introduced.

An exported variable may have a private or inline array type. Its interface
still carries the type needed to read and index the variable, without adding a
qualified name by which the client can declare another variable of that type.
This does not make the private type member accessible.

## IR and QBE lowering

### Storage layout is explicit in the IR

Keep scalar `ir::Ty` for loaded values, arithmetic, arguments, and results.
Add a separate recursive IR storage type for globals and slots. It represents
a scalar or a fixed array and computes the same checked size and alignment
already fixed by semantic analysis. This keeps `ir::Ty` copyable for scalar
instructions and gives the backend enough shape to allocate aggregate storage
without seeing semantic identities.

Module globals and procedure slots use the storage type. Scalar loads and
stores still carry `ir::Ty`; only a selected scalar element can reach them.
An aggregate address is never loaded as one QBE value.

### The IR has checked indexing and byte copy operations

Add one checked index instruction with these facts:

- Destination address temporary.
- Base address.
- Already evaluated INTEGER index value.
- Applicable array length.
- Element stride in bytes.

The applicable length is part of every instruction rather than recovered from
the base allocation. This is necessary for nested arrays and later lets open
arrays supply a dynamic length without changing the address rule.

QBE emission lowers the instruction in this order:

1. Call `oberon_check_index(index, length)` with word arguments.
2. Sign-extend the now-known non-negative index to a QBE long.
3. Multiply it by the long element stride.
4. Add that offset to the base address and define the long address result.

The helper QBE temporaries are derived from the IR destination number and
cannot collide with ordinary value temporaries. No backend type or length
guess is involved.

Add a byte-copy IR instruction containing destination address, source address,
and the constant byte count. QBE lowers it to
`oberon_copy(destination, source, count)` with pointer, pointer, and long
arguments. The byte count is the identical array type's size, including zero.
This avoids expanding a large QBE `blit` into code proportional to the array
size and gives self-assignment the `memmove` semantics stated above.

Declined: representing arrays as pointer-class scalar values. Storage address,
type identity, and copied contents are separate concepts, and conflating them
would turn assignment into aliasing.

Declined: lowering each element copy as a compiler-generated loop. The target
runtime already has the established C memory operation, and a loop would add
control flow without exposing more Oberon behavior.

## Procedures and writable designators

Fixed arrays work in the two structured roles earned by this slice:

- Module variables, including exported read-only variables in clients.
- Procedure-local variables.

Named and inline forms work in both roles. Nested arrays can be selected down
to scalars or assigned at any selected array level.

Fixed-array value and `VAR` parameters remain unsupported until Slice 12,
where structured parameter mutability and the calling convention are handled
together with records. A formal that resolves to a fixed array receives a
stable unsupported diagnostic rather than reaching scalar ABI lowering. Open
array formals remain parser-level unsupported syntax until Slice 14.

An array result receives a permanent source diagnostic because Report 10.1
forbids array and record result types.

Scalar `VAR` parameters and modifying predefined procedures immediately accept
selected scalar elements. Report 9.2 requires their selectors to be evaluated
at parameter substitution time. Address resolution before the call already
provides exactly that behavior.

## What remains unsupported after this slice

- CHAR, BYTE, strings, and their array compatibility rules remain unsupported
  until Slice 11. Arrays in this slice contain the four existing scalar types
  or other fixed arrays.
- Records, record fields, and record layout remain unsupported until Slice 12.
- Fixed-array and record value and `VAR` parameters remain unsupported until
  Slice 12. Structured value-parameter read-only rules therefore do not arrive
  early.
- Pointers, `NIL`, `NEW`, dereference selectors, and heap allocation remain
  unsupported until Slice 13.
- Open array formal parameters and dynamic length passing remain unsupported
  until Slice 14. `LEN` has no dynamic-length form yet.
- Procedure types and type-guard selectors remain in their later slices.
- Strings cannot yet initialize or compare character arrays, and `LEN` does
  not yet accept strings.
- The optional `SYSTEM` module remains outside the core roadmap.

The scalar arithmetic, REAL representation, SET domain, module initialization,
and scalar calling convention from prior slices remain unchanged.

## Changes by file

### src/ast.rs

Add type declarations to modules and procedures. Replace the variable type
placeholder with a source type enum containing a qualified name or a fixed
array expression. Use the same named form for procedure formal and result
types so semantic analysis can reject structured procedure roles deliberately.

Add an index selector containing its expression list and source position.
Extend designator display and expression position handling without trying to
pretty-print complete selector expressions in diagnostics.

### src/parser.rs

Parse the TYPE portion of every declaration sequence between CONST and VAR.
Parse aliases, fixed array constructors, comma-separated dimensions, and
recursively nested element types. Retain explicit unsupported diagnostics for
the other structured type constructors.

Use the general source type parser in type and variable declarations. Formal
types remain the Report's named type form unless they begin with `ARRAY OF`,
which stays unsupported until Slice 14.

Parse bracket selectors and their nonempty expression lists. A comma list stays
one source selector; semantic analysis performs the Report's nested expansion.
Continue to reject caret and type-guard selectors for their later slices.

Add parser tests for an exported type, an alias, inline and nested arrays,
comma dimensions, repeated bracket selectors, malformed lengths, an empty
index list, and missing `OF` or `]` tokens.

### src/sema.rs

Replace the copyable semantic type with basic cases and a shared array
descriptor. Implement explicit identity equality, scalar IR conversion,
layout, and array accessors. Audit every old type copy and scalar-only match;
clone shared types deliberately and reject aggregates before scalar lowering.

Add type symbols and exported interface members. Analyze type declarations in
order at module and procedure scope. Resolve aliases and source array types,
fold and validate every length, expand dimensions into nested descriptors, and
diagnose checked layout overflow or the target object-size limit. Track aligned
slot and global totals so the declaration that would exceed a procedure or
module storage limit receives the diagnostic.

Refactor designator resolution to return an address, resulting type, and
read-only state while applying selectors in order. Lower each array selector
to the checked-index IR instruction. Preserve imported-variable read-only state
through every selector.

Split scalar and aggregate assignment. Scalar assignment keeps the existing
load/store path. Identical array designators emit the byte-copy instruction;
all other aggregate or mixed assignments receive diagnostics.

Install and lower `LEN`. Validate its designator argument without scalar-
loading the array, preserve selector effects, and return the selected array's
fixed length. Extend constant evaluation only for selector forms it can prove
constant.

Make selected scalar designators work through the existing writable-argument
path for scalar `VAR` parameters and modifying predefined procedures. Diagnose
fixed-array procedure formals as not yet supported and array results as illegal.

### src/ir.rs

Add the recursive storage type used by globals and slots while leaving scalar
`Ty` for values. Put size and alignment helpers beside that storage type.
Define the one-GiB target aggregate-storage limit here as the single constant
used by semantic declarations and final program validation.

Add checked index and byte-copy instructions. Both use ordinary addresses;
the checked index defines an address temporary, and the copy produces no
value. No array immediate, aggregate load, aggregate store, array argument, or
array return is added.

### src/qbe.rs

Emit aggregate globals with their complete zero-filled size and alignment.
Emit aggregate locals with a complete aligned allocation, including size zero.
Keep scalar load, store, parameter, result, and operation classes unchanged.

Lower checked index operations to the runtime check followed by QBE long
extension, multiplication, and address addition. Lower aggregate copy to the
runtime copy call. Ensure the runtime check textually precedes every address
calculation in emitted IL.

### runtime/oberon.c

Include the C string facilities and add:

- `oberon_check_index(int32_t index, int32_t length)`, which accepts exactly
  `0 <= index < length` and otherwise prints the stable failure line and exits.
- `oberon_copy(void *destination, const void *source, size_t count)`, which
  delegates to `memmove` and accepts a zero count.

Keep both behind runtime names rather than calling libc directly from generated
code.

### docs/dev/architecture.md

Add an aggregate-layout section. Document array identity, nested representation,
size and alignment, zero-length storage, checked element address calculation,
whole-array copy, and exported type identity.

### src/driver.rs and src/main.rs

In the driver, sum the complete static storage of the dependency-ordered IR
program with checked arithmetic. Reject a build that exceeds the same one-GiB
target static-data limit before writing or passing IL to QBE. No module graph,
lookup, invocation, or linker-order change is needed. `main.rs` does not
change.

## New corpus modules

`tests/corpus/` compiles each module, runs it, and compares standard output.

- `Arrays.Mod` implements a sieve with a named module-level BOOLEAN array. It
  uses `LEN` as the loop bound, writes and reads many dynamic indices, and
  prints the resulting primes or an exact checksum.
- `ArrayElements.Mod` uses INTEGER, BOOLEAN, SET, and REAL element arrays in
  module and procedure-local storage. REAL observations go through `FLOOR` and
  SET observations through `ORD`. It also changes selected scalar elements
  through a scalar `VAR` procedure and the applicable modifying predefined
  operations.
- `ArrayDimensions.Mod` fills and reads both `ARRAY 2, 3 OF INTEGER` and an
  explicitly nested array. Index functions with counters prove left-to-right,
  exactly-once evaluation. `LEN` checks the outer and selected inner lengths,
  including a selected designator with an index function. A local constant and
  a local array length use `LEN` with a valid constant selector, proving the
  separate required-constant path.
- `ArrayCopy.Mod` covers named and inline arrays, whole-array and selected-row
  assignment, and self-assignment. It mutates the source after copying to prove
  the destination owns copied values. It positively assigns through a type
  alias and between anonymous arrays declared in one identifier list. Selected
  row assignment uses effectful destination and source selectors to prove the
  destination is resolved once before the source is resolved once. It declares
  a zero-length array, assigns it, and prints `LEN` as zero.
- `PredefinedShadow.Mod` is extended so a source declaration can shadow `LEN`
  while the existing predefined names continue to work in their scopes.

`tests/corpus/modules/array-api/` supplies the required three-module
cross-module coverage.

- `ArrayTypes.Mod` exports a named fixed array type and initialized variables
  using that type, a private array type, and an inline array type.
- `ArrayFacade.Mod` imports the base module and re-exports an alias of its
  named array type.
- `ArrayApi.Mod` imports both modules, proves the original and re-exported
  names retain one identity, copies an imported read-only array into a local
  array, indexes both, calls `LEN`, mutates the copy, and proves the source did
  not alias it. It also reads, indexes, and applies `LEN` to exported variables
  whose private or inline array types cannot themselves be named by the client.

`tests/errors/` must fail with exact diagnostics.

- `ArrayLengthBad.Mod` covers a negative length, a nonconstant INTEGER length,
  a non-INTEGER length, and an invalid constant expression.
- `ArrayObjectTooLarge.Mod` declares a type whose size fits 64-bit arithmetic
  but exceeds the one-GiB target object limit.
- `ArrayFrameTooLarge.Mod` gives one procedure several individually valid
  locals whose complete aligned frame exceeds the target limit.
- `ArrayModuleStorageTooLarge.Mod` does the same with module globals.
- `ArrayIndexBad.Mod` covers a non-INTEGER index, indexing a scalar, too many
  comma indices, both negative and upper-bound constant indices, and an
  out-of-range constant selector inside `LEN`.
- `ArrayAssignmentBad.Mod` covers separately constructed but structurally
  equal one-dimensional types, separately constructed equal multidimensional
  types, different lengths, different element types, and scalar to array and
  array to scalar assignment.
- `ArrayLenBad.Mod` covers wrong arity, a scalar argument, a type name rather
  than a variable, and a non-designator expression.
- `ArrayResultBad.Mod` declares a function with an array result and pins the
  Report 10.1 diagnostic.

`tests/errors/modules/array-private-type/` proves that a private type name is
absent from a client interface. `tests/errors/modules/array-import-write/`
proves that assignment to an imported array, one of its scalar elements, and
a selected subarray all remain illegal.

`tests/errors/modules/array-program-storage/` uses individually valid modules
whose combined globals cross the program static-data limit. It pins the clear
driver error emitted before QBE runs.

During this incomplete stage, add an exact unsupported diagnostic case for a
fixed-array value and `VAR` formal. Slice 12 removes that case when it adds the
structured calling convention.

`tests/failures/` compiles each module, runs it, and compares stable standard
error after a nonzero exit. Separate roots are required because the first trap
ends the process.

- `ArrayIndexLow.Mod` indexes with a variable containing -1.
- `ArrayIndexHigh.Mod` indexes with a variable equal to the applicable length.
- `ArrayInnerIndexHigh.Mod` fails an inner dimension whose length differs from
  the outer length, proving that the correct nested descriptor reaches the IR.
- `ArrayZeroIndex.Mod` dynamically indexes a zero-length array with zero.
- `ArrayFirstIndexBad.Mod` supplies an invalid first dimension followed by a
  second index expression that would independently trap. The expected array
  bounds line proves the first check runs before the second expression.

Every failure module expects the same `array index out of bounds` line.

## Verification

1. Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
   `cargo test`, and `git diff --check`.
2. Run every new positive binary directly. Confirm zero exit status, empty
   standard error, and byte-for-byte expected standard output.
3. Read the sieve output and confirm it exercised both writes and reads through
   dynamic indices rather than only constant selectors.
4. Inspect `build/ArrayDimensions.ssa`. Confirm each index procedure call
   appears once and in source order. Confirm each call to
   `$oberon_check_index` has the applicable dimension's length and precedes
   the index extension, stride multiplication, address addition, and the next
   dimension's index-expression call. Confirm ordinary `LEN` with a constant
   selector retains its check, while the required-constant uses emit no code.
5. Inspect each runtime-failure IL file. Confirm lengths for the low, upper,
   inner, and zero-length cases are present as word arguments to the check.
   In the bad-first-dimension case, confirm the first check precedes any code
   for the second index expression.
6. Inspect `build/ArrayElements.ssa`. Confirm selected INTEGER, BOOLEAN, and
   SET elements use word loads and stores, selected REAL elements use single
   loads and stores, and every element address is a QBE long.
7. Inspect `build/ArrayCopy.ssa`. Confirm each whole-array assignment passes
   the exact byte size to `$oberon_copy`, self-assignment is retained safely,
   the effectful destination selector precedes the effectful source selector,
   each appears once, and the zero-length assignment still resolves both
   addresses before a zero-count copy.
8. Inspect globals and slots. Confirm a `2 x 3` INTEGER array reserves 24
   bytes, its row stride is 12, all current arrays align to four, and a
   zero-length array reserves zero payload bytes.
9. Inspect the cross-module IL and interface behavior. Confirm the exported
   array global has one data symbol, the client uses that symbol for reads,
   the original and re-exported qualified names retain one type identity, the
   client can select exported values of unnameable private or inline array
   type, and direct private-type lookup fails in semantic analysis.
10. Compile the target-limit errors directly. Confirm oversized types and
    cumulative procedure or module storage stop with source diagnostics, and
    an aggregate multi-module build stops in the driver before QBE. No case may
    leak QBE's `invalid alloc size` or a linker relocation failure.
11. Rebuild unchanged INTEGER, SET, and REAL programs and compare their IL
    with the pre-slice results. Adding storage types and non-copy semantic
    types must not alter scalar code generation.
12. Confirm no newly valid array program can reach an unsupported diagnostic,
    panic, QBE parse error, assembler error, or linker error. Confirm the two
    deliberately deferred array parameter forms stop at their stable source
    diagnostics.

## Order of work

1. Finish and review Slice 9, establish the common green baseline, and retain
   its scalar type and typed-IR decisions while removing `Copy` assumptions.
2. Add source type declarations and array/index AST forms. Parse TYPE sections,
   aliases, array type expressions, dimensions, and bracket selectors with
   parser-focused tests.
3. Add shared semantic array descriptors, identity, checked layout, sequential
   type declarations, exported interface type members, and per-type and
   cumulative target storage-limit checks.
4. Add IR storage types, checked index addresses, aggregate copy, the two
   runtime helpers, their direct QBE lowering, and the driver's aggregate
   static-data validation.
5. Allocate module and local arrays. Resolve nested selectors to scalar and
   subarray addresses, preserving evaluation order and read-only state.
6. Split scalar and aggregate assignment, add `LEN`, and route selected scalar
   elements through existing assignment, `VAR`-actual, and predefined-modifier
   paths.
7. Add the positive, negative, runtime-failure, target-limit, zero-length,
   sequencing, and three-module identity gates. Add a regression module for
   every bug found during implementation.
8. Update the architecture document and complete the verification list.
