# Development roadmap

## Starting point

The compiler already accepts useful INTEGER and BOOLEAN programs. It supports
constants, module variables, assignments, calls to the built-in `Out` module,
relations, short-circuit Boolean operators, `IF`, `WHILE`, and `REPEAT`. It
compiles those programs through QBE and links native executables against the C
runtime and BDWGC.

The test harness compiles and runs positive corpus modules. It also compares
compiler diagnostics for negative modules. The current baseline passes
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and
`cargo test`.

The largest architectural debt is the direct path from the AST to QBE. The
next slice must introduce the typed IR described in the project architecture.
Procedures provide the first language feature that needs the IR to distinguish
addresses, values, storage, calls, and returns.

## Completion target

The core language is complete when every production and semantic rule in the
[May 2016 Oberon-07 Report](../../references/oberon07-report.pdf) is
implemented. The rearranged
[grammar](../../references/oberon07-grammar.ebnf) is the syntax checklist. This
includes every predefined identifier in sections 6.1 and 10.2. Correct source
must either compile or receive a source diagnostic while a slice is still
incomplete. Correct source must never reach `todo!()`, panic, or an internal
QBE error.

The first supported platform remains the current QBE target and native C
toolchain. Portability layers and additional targets remain outside this
roadmap. Implementation-defined choices such as `SET` width, `REAL` precision,
and source module lookup must be documented and tested.

The Report describes `SYSTEM` as an optional and platform-specific module. It
is not part of the core-language completion gate. A possible `SYSTEM` module is
listed separately after the main roadmap.

## Rules for every slice

Each slice must leave behind a compiler that can compile and run more complete
Oberon programs. Infrastructure work belongs in the slice that first needs it.
There should be no standalone rewrite that provides no new end-to-end behavior.

Before implementation, write the slice plan in `docs/dev/plans/`. The plan must
name the Report sections being implemented. It must also state what valid
Oberon remains unsupported after the slice. Review the completed slice in
`docs/dev/reviews/`.

Every slice has the following common gate:

- Add at least one positive module that compiles, links, runs, and has its
  stdout compared byte for byte.
- Add negative modules for the main new static errors. Compare their
  diagnostics exactly.
- Add runtime-failure modules when the slice introduces a dynamic check.
- Add a regression module for every bug found during implementation or review.
- Check constant evaluation against runtime evaluation when an operation can
  occur in both places.
- Inspect generated QBE IL when the slice changes control flow, storage layout,
  or calling conventions.
- Exercise each new type in every legal role that exists at that point. These
  roles include constants, module variables, locals, parameters, and results.
- Consult the Report first. Consult the vendored compilers when the Report
  leaves behavior open or unclear.
- Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and
  `cargo test`.
- Confirm that newly supported correct source cannot reach an unsupported
  diagnostic or an internal panic.

The corpus harness should gain a third class for expected runtime failures in
Slice 4. These tests must check a nonzero exit status and stable stderr. Later
slices should reuse that class for bounds, nil, assertion, and type-guard
failures.

## Slice 4: Typed IR and scalar procedures

This slice introduces user procedures for the current INTEGER and BOOLEAN
language. The typed IR is introduced as part of that work.

Implement module-level proper procedures and function procedures. Support
basic value parameters, basic `VAR` parameters, local constants, local
variables, calls in expressions, final `RETURN` clauses, and direct recursion.
Basic value parameters are local variables and may be assigned as required by
Report 10.1. A `VAR` actual must be a writable designator.

Add `ir.rs` between semantic analysis and QBE emission. The first IR only needs
the operations earned by this slice. It must represent typed values, addresses,
loads, stores, branches, direct calls, and returns. QBE emission must stop
resolving AST names or making type decisions.

Move module variables out of the module initializer's stack frame. Emit them
as QBE data objects so procedures can access them. Allocate procedure locals in
their own activation records. Add explicit runtime checks for INTEGER `DIV`
and `MOD` by zero.

The gate programs should include a recursive function, a proper procedure that
updates a `VAR` parameter, and a procedure that reads and writes a module
variable. Negative tests should cover a non-variable `VAR` actual, a missing or
ill-typed function result, a proper procedure used as a value, and a function
called as a statement. A runtime test must pin division by zero.

Slice 4 is complete when all existing programs pass through the typed IR. No
direct lowering from the AST to QBE may remain.

## Slice 5: Nested procedures

This slice completes procedure nesting for the scalar language. It adds local
procedure declarations at arbitrary depth, lexical shadowing, and direct
recursion by a nested procedure.

Report 10 defines a procedure's visible environment as its formal parameters,
the objects declared in its own body, and the objects declared in the module
scope. The procedure identifier is also visible in its body for direct
recursion.

Semantic analysis must keep a separate local scope for each procedure. Name
lookup searches that local scope and the module scope. The declaration sequence
adds each local procedure to its owner's scope before the owner's statement
sequence is analyzed.

The gate must compile procedures nested at least three levels deep. Each
procedure must use its own parameters and locals. Each must also access the
same module variable. An outer procedure must call its local child. A nested
procedure must recurse directly. Negative tests must diagnose references to a
variable, parameter, constant, or procedure declared in a different procedure
body. The IL review must verify the path-mangled names and declared parameter
lists of the generated functions.

## Slice 6: Remaining scalar control and predefined operations

This slice completes control flow for INTEGER and BOOLEAN. Add `FOR` and
INTEGER `CASE`, including constant labels and label ranges. Reject overlapping
case ranges. Resolve unclear `FOR` limit and step behavior against Project
Oberon before implementation and pin the choice with tests.

Add the predefined operations that apply to the current types. This includes
INTEGER forms of `ABS`, `ODD`, `LSL`, `ASR`, and `ROR`. It also includes `INC`,
`DEC`, `ASSERT`, and the BOOLEAN form of `ORD`. Constant folding and runtime
lowering must share the same rules.

The gate must cover positive and negative `BY` values, an empty loop, case
ranges, and a case with no matching label. It must show that the `FOR` control
expression behavior matches the chosen reference behavior. Runtime tests must
pin failed `ASSERT` calls. Negative tests must cover a nonconstant or zero step,
an invalid control variable, and overlapping labels.

At this point the single-module scalar language is complete except for the
remaining basic types.

## Slice 7: Real modules, exports, and initialization

This slice replaces the built-in module-name special case with a real module
graph. Compiling a root module must discover its imports, analyze dependencies,
emit every required module, and link one executable.

Support export marks, import aliases, exported constants, exported variables,
and exported procedures. Imported variables are readable but not assignable.
Only exported declarations are visible to a client. Detect missing modules,
module-name and file-name mismatches, duplicate imports, and import cycles.

Choose and document a simple source lookup rule. Start by searching the root
module's directory and the bundled library directory. Reparse dependency
sources on each build rather than adding an interface cache before it is
needed.

Emit one initializer per module. Execute each initializer once after all of its
dependencies have initialized. Keep runtime-backed `Out` available through the
same import and symbol-interface machinery until the library slice replaces
the current special implementation.

The gate must include a three-module program, an aliased import, and a diamond
dependency. Observable initialization output must prove dependency-first and
exactly-once execution. Cross-module negative tests must cover private names,
writes to exported variables, and cycles.

Every later type slice must add at least one cross-module test. This keeps type
identity, visibility, and ABI behavior honest as the language grows.

## Slice 8: SET values and operations

This slice adds the `SET` basic type, set constructors, ranges, union,
difference, intersection, symmetric difference, complement, equality, and
`IN`. Add `INCL`, `EXCL`, and the SET form of `ORD`.

Choose and document the implementation's set limit. A 32-bit representation is
the natural first choice for the current target. Diagnose out-of-range constant
elements. Add a runtime check for out-of-range dynamic elements rather than
allowing target shift behavior to define the language.

The gate must compare folded and runtime results for every set operator. It
must cover empty and full sets, reversed ranges, dynamic ranges, and membership
at both limits. Negative and runtime tests must cover operand mismatches and
out-of-range elements.

## Slice 9: REAL values and numeric conversions

This slice adds `REAL` literals, variables, constants, parameters, results, and
arithmetic. Add real division and numeric ordering. INTEGER and REAL operands
must not mix without an explicit conversion.

Choose and document the `REAL` representation after checking Project Oberon
and the QBE ABI. Use that representation consistently in constant folding,
IR types, data layout, calls, and QBE operations. Avoid a general target-type
abstraction while there is only one target.

Complete the generic REAL form of `ABS`. Add `FLT`, `FLOOR`, `PACK`, and `UNPK`.
Implement the variable-argument effects specified for `PACK` and `UNPK` in
Report 10.2.

The gate must compare folded REAL expressions with runtime expressions through
`FLOOR` or another exact observation. Cover negative flooring, exponent
literals, procedure calls with REAL values, and values stored in module data.
Negative tests must cover implicit INTEGER and REAL mixing and invalid
conversion arguments.

## Slice 10: Fixed arrays and checked indexing

This slice introduces type declarations, fixed array types, inline array types,
multidimensional arrays, and index selectors. Array lengths are positive
constant INTEGER expressions. A declaration with several dimensions is
represented as nested arrays, as required by Report 6.2.

Define array size, alignment, and element address calculation in semantic
types and the IR. Support arrays in module storage and procedure-local storage.
Support whole-array assignment for identical array types. Do not add open
arrays in this slice. Add `LEN` for fixed arrays.

Every index operation must carry the applicable length into the IR. Emit a
runtime bounds check before address calculation. Evaluate each index expression
once.

The gate should include a sieve or another program that makes substantial use
of indexed storage. Add a multidimensional program and a whole-array copy test
that proves the destination does not alias the source. Runtime tests must cover
negative and upper-bound indices. Negative tests must cover invalid lengths,
non-INTEGER indices, incompatible array assignment, and indexing a non-array.

## Slice 11: CHAR, BYTE, strings, and character arrays

This slice adds the `CHAR` and `BYTE` basic types. It also adds string literals,
string constants, and the special compatibility rules between strings, CHAR,
and character arrays.

Store character arrays with the terminating null character required by Report
9.1. Implement assignment from a fitting string and reject a string that is too
long. Add character and character-array comparisons. Extend scalar `CASE` to
CHAR labels and ranges.

Implement the compatibility between BYTE and INTEGER. Decide and document how
dynamic values are checked when stored in BYTE. Add the CHAR form of `ORD` and
add `CHR`. Add runtime-backed character and string output so corpus programs can
observe exact values.

The gate must cover hexadecimal single-character strings, quoted one-character
strings, embedded character arrays, lexical character-array ordering, and
boundary BYTE values. It must distinguish a string assignment from an array
alias. Runtime tests must cover any chosen BYTE range check. Negative tests
must cover an oversized string and incompatible string use.

## Slice 12: Records and structured parameters

This slice adds record types without extension. Support named and inline
records, field declarations, field selection, nested records, arrays of
records, records containing arrays, and whole-record assignment.

Define field order, padding, alignment, and total size in one layout function.
Represent field addressing explicitly in the IR. Record equality remains
illegal because the Report does not define it.

Extend procedures to accept fixed arrays and records. Enforce exact type
identity for `VAR` structured parameters. Structured value parameters are
read-only, including all of their elements and fields. Resolve their aliasing
behavior against the Report and Project Oberon. Pin that behavior with a test
where one actual is also passed through a `VAR` parameter.

Parse and retain field export marks. Enforce them when a record type crosses a
module boundary.

The gate must include nested field selection, record and array copies, and both
value and `VAR` structured parameters. A copy test must prove that later source
mutation does not change the assigned destination. Negative tests must cover
unknown or private fields, assignment through a structured value parameter,
and incompatible record types.

## Slice 13: Pointers, NIL, and garbage-collected allocation

This slice adds pointers bound to record types, the permitted forward reference
to a record in the same scope, `NIL`, `NEW`, explicit dereference, and implicit
pointer dereference during field selection. Add pointer assignment, equality,
parameters, and results.

Use `oberon_alloc` for objects that may contain pointers. Use
`oberon_alloc_atomic` only when the complete object layout proves that it
contains no pointers. Keep all allocator knowledge behind those runtime
wrappers.

Emit a runtime nil check before every dereference. The check must cover both an
explicit caret and the implicit dereference in `p.field`.

The gate must compile linked-list and tree programs. One program should retain
objects only through generated-code pointers while allocating enough temporary
objects to exercise the collector. Runtime tests must cover explicit and
implicit nil dereferences. Negative tests must cover non-record pointer bases,
illegal forward references, and incompatible pointer assignment.

## Slice 14: Open arrays and length passing

This slice implements one-dimensional and multidimensional open-array formal
parameters. Define a small ABI that passes the data address and the required
lengths. Make those lengths explicit in the IR so indexing and `LEN` do not
depend on backend guesses.

Support fixed arrays and compatible open arrays as actual parameters. Support
both value and `VAR` open arrays. Value open arrays remain read-only. Implement
the open-array assignment rule for equal base types. Allow strings where the
Report's character-array compatibility rules permit them.

The gate must include sum, mutation, and search procedures over arrays of
several lengths. Add a multidimensional procedure that observes each length and
checks each dimension. Include a string-processing procedure over an open
character array. Runtime tests must prove that bounds checks use the actual
length. Negative tests must cover rank, base-type, and mutability mismatches.

## Slice 15: Record extension and dynamic type operations

This slice implements record extension and the inherited extension relation for
pointers. Lay out a derived record with its base record as a stable prefix. Add
runtime type descriptors for allocated records and for record variables passed
polymorphically.

Implement extension assignment to a base record. Allow a derived record as the
actual argument for a base-record `VAR` parameter. Implement `IS`, designator
type guards, and the record and pointer forms of `CASE`. Narrow the case
variable only within its matching arm.

Emit a runtime failure for a false type guard. A type test must remain a pure
Boolean operation. Resolve the Report's undefined `NIL IS T` case explicitly
and document the chosen behavior.

The gate must define a hierarchy in one module and use it from another. It must
cover successful and failed tests, guards, polymorphic `VAR` calls, and type
case arms. It must prove inherited and private field visibility. Negative tests
must reject unrelated types and illegal test or guard subjects. A runtime test
must pin a failed guard.

## Slice 16: Procedure types and indirect calls

This slice adds procedure types, procedure variables, procedure parameters,
assignment of eligible procedures, `NIL`, equality, and indirect calls. Support
both proper and function procedure signatures. Require exact parameter and
result types.

Only globally declared procedures may become procedure values. Predefined and
nested procedures remain ineligible, as required by Report 6.5 and 10.1.
Results may not be arrays or records.

Represent an indirect callable value explicitly in the IR. Extend the existing
calling convention instead of creating a second lowering path. Add a runtime
failure for calling a `NIL` procedure value.

The gate must include callbacks, a function procedure variable, and a procedure
value imported from another module. It must cover equality with `NIL` and
passing an open-array procedure signature. Negative tests must cover signature
mismatches and attempts to store nested or predefined procedures.

After this slice, every core grammar production has an implementation.

## Slice 17: Portable standard modules

This slice replaces the temporary `Out` implementation with bundled standard
modules. Define the supported portable library profile before implementation.
The initial profile should include `Out`, `In`, `Math`, `Strings`, and `Files`,
using the vendored OBNC library interfaces as a compatibility reference.

Write as much of each module as practical in Oberon. Keep operating-system and
C-library calls in the small C runtime. Expose those calls only through private
compiler and runtime bindings. Do not add a foreign-function extension to the
language.

The module loader must find bundled modules through the normal lookup rule.
User modules and bundled modules must follow the same export, type, and
initialization rules.

The gate must compile multi-module applications that perform formatted output,
token input, string manipulation, mathematical operations, and file round
trips. Tests must use temporary files and deterministic input. Each public
library procedure needs at least one behavior test and one relevant boundary
test.

Interactive input and graphics modules can follow later. They are not part of
the portable profile because their behavior depends on a terminal or windowing
system.

## Slice 18: Conformance closure and release gate

This slice is an audit rather than a feature bucket. Build a matrix from every
grammar production, predefined identifier, type-compatibility rule, selector,
operator, statement, parameter mode, and module visibility rule in the Report.
Link every row to one or more tests.

Run suitable programs from the vendored compiler suites without copying their
implementation code. Add focused corpus modules for uncovered behavior. When
the reference compilers disagree, record the Report-based decision in a test
comment or development note.

Audit malformed source at the lexer, parser, and semantic boundaries. Every
case must produce a source diagnostic and a nonzero exit. Audit every dynamic
check for stable stderr and a nonzero exit. Search for remaining `todo!()`,
unsupported diagnostics, and backend assumptions that correct source can
reach.

Run the complete suite in debug and release modes. Inspect representative QBE
IL for each storage class and calling convention. Exercise module
initialization, recursion, deep nesting, aggregate copies, GC retention, and all
runtime failures together in larger programs.

The release gate is closed only when the conformance matrix has no unexplained
gaps. The final review must list every implementation-defined choice and every
known limitation. A limitation that rejects a valid core Oberon-07 program is a
release blocker.

## Optional work after core completion

The optional `SYSTEM` module needs its own plan. That plan must define the
address representation before implementing `ADR`, `SIZE`, `BIT`, `GET`, `PUT`,
and `COPY`. The current 32-bit INTEGER and 64-bit native pointer combination
makes this a real target decision rather than a small library task.

Additional native targets, interface-file caching, incremental compilation,
self-hosting, a JIT, and advanced optimization remain non-goals. They should not
shape the core compiler unless the project goals change.
