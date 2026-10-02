# Slice: Open arrays and length passing

## Context

After Slice 13 the compiler has fixed arrays with exact layout, checked indexing, selected-row addressing, whole-value copies, structured value and `VAR` parameters, strings and bounded character-array relations, pointers into garbage-collected records, and module interfaces that preserve type identity and procedure signatures. Every array length is currently stored in a semantic type and emitted as an immediate. A structured formal currently needs one pointer-class argument because its complete shape is known from that type.

This slice adds one-dimensional and multidimensional open-array formal parameters. An open formal has no declared length of its own, so a call passes the actual array's address and one dynamic length for each open dimension. Those lengths remain explicit through semantic place resolution and the typed IR. Indexing, `LEN`, character-array comparison, open-array forwarding, and checked assignment all use the same values rather than recovering shape from an allocation or relying on a backend convention.

Report sections implemented: section 5 for the boundary between fixed `LEN` expressions and dynamic open-array lengths in constant contexts; section 6.2 for array lengths, element types, and index domains as applied to open formals; section 8.1 for indexing open arrays and multidimensional selector abbreviation; section 8.2.4 for relations involving open character arrays; section 9.1 for read-only structured value parameters, string assignment to an open character array, and assignment from an open array to an array of equal element type; section 9.2 for substituting and evaluating array actual parameters; section 10.1 for `FormalType = {ARRAY OF} qualident`, arbitrary actual lengths, and value and `VAR` open formals; section 10.2 for dynamic `LEN`; and section 11 for exported procedures with open-array formals.

Implementation starts from the current Slice 13 working tree. Its common gate is green: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` all pass, including the pointer, nil-check, and garbage-collection corpus.

## Language and representation decisions

### Open arrays exist only as formal parameter types

The grammar permits an open array only in `FormalType`, as one or more `ARRAY OF` prefixes followed by a qualified type name. It does not permit an open array in a type declaration, variable declaration, record field, pointer base, or result type. The AST therefore records the open prefixes on a formal parameter section instead of adding a general source type constructor that ordinary declarations could accidentally accept.

Each prefix is one open dimension. `ARRAY OF INTEGER` has one. `ARRAY OF ARRAY OF INTEGER` has two. `ARRAY OF Row`, where `Row` is a named fixed array type, has one open dimension followed by the complete fixed `Row` type. The prefixes are not flattened into a rank and a leaf type because the boundary between open dimensions and a named fixed-array base matters to compatibility, constant `LEN`, element stride, and procedure signatures.

The semantic type has an open-array form so procedure symbols and module interfaces can carry the formal exactly. It has an element type but no storage size, alignment, declared length, or constructor identity. Two formal types are the same signature type when they have the same open-prefix structure and reach the identical qualified base type. Open arrays never reserve globals or slots and can never reach ordinary layout functions.

Declined: representing an open array as a fixed array with a sentinel length such as `-1`. A sentinel makes every existing size, alignment, storage, constant-index, and object-limit path responsible for excluding a non-type length. A distinct semantic form keeps the invariant literal: fixed arrays own storage and a non-negative declared length; open arrays describe borrowed storage and obtain their lengths from a call.

### Actual compatibility is recursive across the open prefixes

For each open prefix in the formal, the actual must supply one array dimension of arbitrary length. Compatibility then recurses into the element type. When the formal reaches its qualified base type, the actual must reach the identical type. This accepts a fixed array, another compatible open formal, or a selected subarray whose remaining dimensions match. It rejects a rank mismatch, a different terminal basic or named type, and a separately constructed array or record type that merely prints the same.

For example, `ARRAY OF Row` accepts a fixed array whose elements have the identical named type `Row`; the fixed dimensions inside `Row` remain part of that identity. `ARRAY OF ARRAY OF INTEGER` accepts any two-dimensional fixed INTEGER array and can be forwarded from another two-dimensional open formal. It also accepts an actual of formal type `ARRAY OF Row` when `Row` is a fixed INTEGER array: the formal's outer open prefix consumes the actual's open dimension, and its inner open prefix consumes `Row`'s fixed dimension. Selecting one row from any of these produces a one-dimensional array that can be passed to `ARRAY OF INTEGER`.

A structured value actual remains a designator and may be read-only. A `VAR` open actual must be a writable designator. An imported array, a structured value parameter, or a selection from either may be passed to a value open formal but not to a `VAR` open formal. Resolving a selected actual evaluates every selector once, before the procedure starts, as Report 9.2 requires.

A string literal or named string constant is the one non-variable actual accepted for a one-dimensional value formal of `ARRAY OF CHAR`. The call materializes the existing literal data object and passes its address with a length equal to the character count plus the appended null. A string is never accepted for a `VAR` formal, for a non-CHAR element type, or for more than one open dimension. This follows Project Oberon and OBNC and gives strings the parameter path deliberately deferred by Slices 11 and 12.

### The call ABI is one address followed by the open lengths

Each open-array source parameter expands to one QBE `l` argument containing the data address, followed immediately by one QBE `w` INTEGER length for each open prefix, ordered from the outermost dimension to the innermost. Value and `VAR` open formals have the same machine shape. Their difference remains semantic: a value formal is read-only and a `VAR` formal is writable.

Each open prefix in the callee consumes the corresponding dimension of the actual and passes that dimension's length. A fixed actual therefore contributes declared immediates. An all-open actual contributes incoming dynamic values. A mixed actual can contribute both: forwarding `ARRAY OF Row` to `ARRAY OF ARRAY OF INTEGER`, where `Row` is fixed, passes the incoming outer length followed by `Row`'s declared length. A selected subarray contributes only the shape that remains after its selectors. A string contributes its literal address and its character count plus one. A parameter section with several names expands each named formal independently; no length is shared merely because the source names appeared in one section.

The address and all lengths appear as ordinary, explicit IR parameters and call arguments. No packed descriptor is allocated, no hidden stack object is created, and QBE does not infer anything from an address. Procedure symbols and module interfaces continue to count source parameters for arity diagnostics while lowering expands each open formal into its machine arguments.

Every length is a non-negative 32-bit INTEGER. Generated calls are the only source of a hidden length: fixed array types were already checked against the target object-size limit, strings have compiler-known lengths, and forwarding preserves an incoming length. Source code cannot fabricate or mutate a hidden length.

Declined: passing one flattened element count for a multidimensional array. `LEN(a)`, `LEN(a[0])`, and each dimension's bounds check need different lengths. Reconstructing them from a product is impossible in general, and flattening would erase the Report's nested-array meaning.

### A place carries the lengths and strides needed by its array dimensions

The runtime designator walk extends its place description with array-shape metadata. A fixed dimension contributes an immediate length. An open dimension contributes its incoming parameter value. Indexing consumes the current dimension's length, emits the check with that exact value, advances to the element type, and leaves the remaining shape attached to the selected place. Field selection and pointer dereference preserve or reconstruct the fixed shape of any array type they reach.

An open multidimensional array also makes some element strides dynamic. For `a: ARRAY OF ARRAY OF INTEGER`, the address of `a[i]` advances by `i * LEN(a[0]) * size(INTEGER)`, while the address of `a[i, j]` advances its second step by `j * size(INTEGER)`. With three open dimensions, the outer stride is the product of both remaining lengths and the terminal element size. A named fixed-array base contributes its already known complete size: the outer stride of `ARRAY OF Row` is `size(Row)`, not another hidden value.

The IR index instruction changes its length operand from a fixed `i32` to an ordinary INTEGER value. Its stride becomes an explicit product of a fixed byte factor and zero or more remaining open-dimension lengths. Fixed indexing supplies an immediate bound and no dynamic stride factors, so it should emit byte-for-byte equivalent QBE. Open indexing supplies a dynamic bound and, where needed, dynamic factors. QBE widens each non-negative factor to `l` before multiplying. In every case the bounds check precedes stride calculation, address formation, and evaluation of the next dimension's expression.

The products cannot exceed the storage behind the actual. Every fixed actual was checked against the target object-size limit when its type was constructed, a forwarded open actual preserves those same dimensions, and a string has one dimension. Source code cannot forge a length. Dynamic stride multiplication therefore relies on a compiler-maintained call invariant rather than adding a second runtime shape check.

A constant index into a fixed array keeps the existing source check. An index into an open dimension always uses the actual runtime length, even when its source expression is constant. This matches Project Oberon and OBNC: the same constant may be valid for one call and out of bounds for another. A zero-length actual therefore fails every executed index through the ordinary runtime check without special address behavior.

Selected array actuals reuse the place already resolved for the call. Their address and remaining lengths are read from that one result, so an index procedure in `P(table[next()])` runs once even though the call passes both data and shape.

### `LEN` returns the current dimension's fixed or dynamic length

Executable `LEN` resolves its designator exactly as it does now. It returns the current array place's length, which is an immediate for a fixed dimension and an incoming value for an open dimension. `LEN(a[i])` still evaluates and checks `i` before returning the selected inner dimension's length. For a multidimensional open formal, `LEN(a)`, `LEN(a[0])`, and `LEN(a[0, 0])` therefore observe the separately passed outer, middle, and inner lengths.

In a required constant context, `LEN` folds only when the selected dimension is fixed. `LEN(a)` for an open formal is not a constant, and neither is `LEN(a[0])` when that selected inner dimension is open. If `a` has formal type `ARRAY OF Row` and `Row` is a fixed array, `LEN(a[i])` may still fold to `Row`'s declared length through the existing type-only selector walk: the open outer selector is type-checked but not executed, and the answer does not depend on the actual length or on `i`.

The constant designator walks learn the open-array form so they diagnose bad selectors and non-INTEGER indices without demanding a runtime length. They do not diagnose a constant index against an open bound because that bound is unavailable until the call.

### Value open arrays are borrowed read-only views

Like every existing structured value parameter, a value open array is an address into caller-owned storage and no callee-side copy occurs. The entire selected value is read-only, including nested elements, fields, and records reached through pointers. Passing the same actual to a value open formal and a `VAR` open formal therefore preserves the aliasing rule pinned in Slice 12: mutation through the `VAR` view is immediately visible through the value view.

An open formal can be forwarded without allocating or copying. A value view stays unusable as a `VAR` actual even when forwarded through several calls. A writable `VAR` view stays writable. The machine ABI does not encode this distinction, so semantic read-only propagation must remain attached to every derived place.

### Open-array assignment is a checked prefix copy

Report 9.1's open-array exception is implemented in the direction used by Project Oberon and OBNC: an open array source may be assigned to a fixed array destination when their immediate element types are identical. The source length is dynamic and the destination capacity is fixed. The compiler checks that the source length is no greater than the destination length, then copies exactly `source length * element size` bytes. Elements beyond the copied prefix of a larger destination remain unchanged. A zero-length source performs a checked zero-byte copy.

The immediate element-type rule makes a selected row from a multidimensional open formal assignable to a fixed row of the matching type. It does not make a whole `ARRAY OF ARRAY OF INTEGER` formal assignable to a fixed two-dimensional array, because the source's immediate element is itself open while the destination's immediate element is fixed. Source code can select and copy rows explicitly. This is the behavior OBNC exercises and avoids inventing a rectangular bulk-assignment rule absent from the Report.

General assignment into an open destination is not added. A fixed source cannot be assigned to an open formal, and one open formal cannot be assigned to another. Project Oberon and oberonc reject those forms, and the Report states only that an open array may be assigned to an array. This keeps an open formal a borrowed view rather than a resizable value.

String assignment is the independent Report 9.1 exception. A writable one-dimensional open `ARRAY OF CHAR` may receive a string. The source count is the character count plus its null terminator, the destination capacity is the incoming length, and the copy proceeds only if the complete count fits. A fixed character-array destination retains its existing compile-time fit diagnostic.

The IR gains an explicit array-copy length check and a dynamic byte-count copy form. QBE emits the check before it widens the accepted source length, multiplies by the fixed element stride, and calls `oberon_copy`. The runtime function `oberon_check_array_copy(source_length, destination_length)` prints `array assignment exceeds destination length` and exits when the source is longer. Oversized open-source assignment and oversized string assignment to an open character array use this same stable failure.

### Open character arrays retain the existing bounded text rules

An open `ARRAY OF CHAR` is a character array for all six relations. Its bound passed to `oberon_str_cmp` is its dynamic length. A fixed character array still contributes its declared length, and a string contributes its character count plus the null. The existing comparison therefore needs no runtime change; semantic lowering only stops assuming that every character-array bound is an immediate.

This permits a string-processing procedure to compare a value open character array with strings, fixed character arrays, and another open character array without reading past any actual. A value string actual is represented by the same literal address and terminator-inclusive length used by `LEN` and by parameter passing.

## Semantic types and module interfaces

The semantic type gains an open-array form with an element type and no layout. Helpers distinguish fixed arrays, open arrays, and either kind of array. Character-array recognition returns a length description rather than only an `i32`, so assignment and relations can accept a fixed immediate or a dynamic value. Type display prints `ARRAY OF T` recursively, which keeps the open rank and the fixed base's existing structural display visible in diagnostics. Fixed array identity still comes from its shared descriptor; this slice does not add declared names to fixed-array diagnostics.

Procedure symbols and interface members continue to carry source formal types. Cloning an interface shares every named terminal type while preserving the open wrappers around it, so a client checks the same rank and identity rules as the declaring module. No new interface member kind or serialized ABI description is needed.

Variable symbols and resolved places retain the runtime lengths belonging to an open formal and derive the applicable row-major strides from the remaining lengths and fixed base size. Ordinary fixed variables need no stored hidden values; their metadata can be derived from their semantic types. Reference-actual lowering returns the complete place rather than discarding everything except its address and type, because forwarding an open or selected array also needs its remaining shape.

Assignment compatibility gains named outcomes for open-source prefix copy and string-to-open-character-array copy. These are statement-only outcomes. Scalar value arguments and function results must not inherit them, and fixed structured formals keep their exact-identity actual rule.

## IR and QBE lowering

An open formal expands in `ir::Proc.params` to one existing reference parameter and one existing INTEGER value parameter per open dimension. A call expands in `ir::Inst::Call.args` to the matching reference and INTEGER values. The IR therefore exposes the complete ABI with no backend-only hidden arguments and needs no descriptor or new parameter class.

`ir::Inst::Index` takes an INTEGER value as its length and an explicit stride description consisting of a fixed byte factor and any dynamic inner lengths. QBE prints either the old immediate or the dynamic temporary in `oberon_check_index`, widens the index and dynamic stride factors to `l`, multiplies the stride in row-major order, and then forms the address. A fixed array has no dynamic factors and keeps the current lowering.

The IR adds `CheckArrayCopy` with source and destination INTEGER lengths. It also adds a copy whose byte count is derived from an INTEGER element count and a fixed element stride, or equivalently extends the current byte-copy count with an explicit dynamic form. QBE lowers the dynamic form by sign-extending the compiler-supplied non-negative count to `l`, multiplying by the stride in `l`, and passing that byte count to `oberon_copy`. Existing fixed array, record, and string copies keep their present constant counts and emitted IL.

No open-array object appears in QBE data or a procedure's slots. A pointer-class argument may be an interior address selected from a heap record or array. Slice 13 already enables arbitrary interior pointers before collector initialization, so such an address remains a conservative root across calls and allocation without a new GC rule.

## Procedures and predefined operations

Open arrays work for proper procedures and function procedures, at module scope and every supported nesting depth. They work in exported procedures and direct recursion. A procedure can mix scalar, fixed structured, pointer, value open, and `VAR` open parameters; source arity and diagnostic numbering still count each declared name once despite ABI expansion.

`LEN` is the only predefined operation whose result changes. Indexing, whole-source assignment, string assignment, and character-array relations use dynamic lengths through their existing language rules. Other predefined operations see selected scalar elements or fields and need no open-array special case.

Every terminal type available after Slice 13 can be an open array element: basic types, pointers, fixed arrays, and records. Compatibility at that terminal remains exact identity apart from the string actual exception for `CHAR`; pointer value assignment compatibility and BYTE-to-INTEGER scalar compatibility do not turn arrays of those types into compatible arrays.

## What remains unsupported after this slice

- Record extension, inherited pointer compatibility, extension assignment and parameter passing, `IS`, type guards, and the record and pointer forms of `CASE` remain in Slice 15.
- Procedure types, procedure values, indirect calls, and comparison with procedure `NIL` remain in Slice 16. Slice 16 must include open-array shapes and value-versus-`VAR` mode when it compares procedure signatures.
- Portable `Out.String`, `Strings`, and the other standard modules remain in Slice 17. This slice supplies the open-character-array language and ABI they need but does not add library APIs early.
- Array and record function results remain forbidden by Report 10.1. Open arrays remain formal types only; there are no slices, dynamic arrays, ragged arrays, or heap-allocated array values in the language.
- Runtime failures still carry no source position. Oversized open-array assignment receives the same stable process-level diagnostic shape as existing bounds, nil, and range checks.
- The optional `SYSTEM` module remains outside the core roadmap.

Fixed array identity and layout, record layout, pointer representation, garbage-collected allocation, module initialization, and every scalar calling class remain unchanged.

## Changes by file

### src/ast.rs

Record the ordered `ARRAY OF` prefixes on each formal parameter section, with enough position information for parser and semantic diagnostics. Keep the qualified base as the existing named type expression. Ordinary `TypeExpr` remains unable to represent an open array in a variable or type declaration.

### src/parser.rs

Parse `FormalType = {ARRAY OF} qualident`, preserving every prefix and allowing zero prefixes for the existing named formal. Add shape tests for one, two, and three open dimensions; an open prefix before a named fixed-array base; mixed value and `VAR` sections; several names sharing one section; exported and nested procedures; and malformed missing `OF` or base names. Remove only the open-formal assertion from the unsupported-construct test; procedure types and record extension remain stable.

### src/sema.rs

Add the open-array semantic form, structural open compatibility, display, array-kind helpers, and the invariant that it has no storage layout. Resolve formal types through a dedicated path that wraps the named base in the recorded open prefixes, while ordinary type resolution continues to reject anything without a fixed layout.

Expand each open formal into an address parameter and its outer-to-inner INTEGER length parameters. Bind the source name to one read-only or writable place holding that address and those lengths. Extend procedure symbols and module interfaces with the open formal type, then expand calls from fixed arrays, open arrays, selected subarrays, and strings while preserving source argument numbering and left-to-right, exactly-once evaluation.

Carry fixed or dynamic array shape through places. Make indexing consume the current length and compute its stride from the remaining dimensions, make executable `LEN` return the current length, and update the executable and constant selector walks for mixed open and fixed dimensions. Keep required-constant `LEN` available only when the selected dimension is fixed.

Extend open character arrays through string actuals, bounded relations, and writable string assignment. Add open-source assignment to a fixed destination with immediate element identity, the capacity check, dynamic byte count, and unchanged destination tail. Preserve the read-only rule for value formals in assignment, modified predefined operations, and `VAR` substitution.

Ensure every invalid rank, base type, mutability, assignment direction, and constant use produces a source diagnostic before IR construction, including after earlier errors in the same formal list or call.

### src/ir.rs

Change the index length to an INTEGER value and make its stride an explicit fixed factor times any dynamic inner lengths. Add the explicit array-copy length check and the dynamic element-count copy representation. Reuse reference and INTEGER value parameters and call arguments for the open-array ABI; no open-array storage form exists.

### src/qbe.rs

Emit dynamic index lengths and row-major stride products, expanded address-and-length procedure signatures and calls, the array-copy check, and the widened dynamic byte-count copy. Keep fixed index and fixed copy output unchanged when their operands are immediate.

### runtime/oberon.c

Add `oberon_check_array_copy(int32_t source_length, int32_t destination_length)`. It exits with `array assignment exceeds destination length` when the source is longer. Keep copying in `oberon_copy`; the checker does not combine policy with memory movement.

### agents/architecture.md

Add open formal representation, recursive compatibility, the address-plus-lengths ABI, source-to-machine parameter expansion, place shape metadata, dynamic row-major stride derivation, dynamic indexing and `LEN`, string actuals, open character-array comparison, value and `VAR` mutability, checked open-source assignment, and the constant-expression boundary. State that open arrays own no storage and that a selected heap address relies on Slice 13's interior-pointer collector setting.

### Files intentionally unchanged

`src/lexer.rs` already recognizes `ARRAY` and `OF`. `src/driver.rs` and `src/main.rs` already emit and link arbitrary IR procedure signatures. The corpus harness already supports positive, compile-error, and runtime-failure modules. `Cargo.toml` needs no dependency change.

## New corpus modules

`tests/corpus/` compiles each module, runs it, and compares standard output.

- `OpenArrays.Mod` defines sum, search, and mutation procedures over `ARRAY OF INTEGER`. It calls them with zero-, one-, and several-element fixed arrays; forwards value and `VAR` open formals through nested and recursive procedures; mixes scalar, pointer, fixed structured, and open parameters; and passes the same array as value and `VAR` to prove the existing aliasing rule. One procedure declares `a, b: ARRAY OF INTEGER` in the same formal section, receives actuals of different lengths, and prints and indexes both lengths independently. A selected-row actual uses an index procedure whose counter proves the selector is evaluated once.
- `OpenArrayDimensions.Mod` passes two- and three-dimensional fixed arrays with different inner lengths to matching open formals. It places distinctive values where an incorrect fixed or flattened stride reads the wrong element, prints `LEN` at every dimension, uses comma and repeated-bracket indexing, forwards the full open array and selected rows, mutates through a three-dimensional `VAR` formal, and covers `ARRAY OF Row` where `Row` is a named fixed array. It forwards that mixed open/fixed formal to `ARRAY OF ARRAY OF INTEGER`, proving the next call passes one dynamic and one fixed length. A constant inside the mixed procedure folds `LEN(a[i])` from the fixed `Row` type without evaluating `i`.
- `OpenArrayAssignment.Mod` assigns one-dimensional open sources of lengths zero, exact capacity, and shorter than capacity into fixed destinations with the identical element type. It proves only the source prefix changes, covers records and a selected row, and performs an aliased self-copy through the existing `memmove` wrapper. It also assigns fitting strings through writable open character arrays.
- `OpenArrayStrings.Mod` passes literals, named string constants, and fixed character arrays of several lengths to a value open character-array procedure. The procedure observes the terminator-inclusive `LEN`, searches without crossing the bound, and compares its formal with strings, fixed arrays, and another open character array. A `VAR` procedure mutates characters and assigns a fitting string.

`tests/corpus/modules/open-array-api/` supplies the cross-module coverage.

- `OpenArraySupport.Mod` exports a named record element type and procedures with one-dimensional, multidimensional, value, and `VAR` open formals. It forwards formals internally and exports a function with an open parameter and scalar result.
- `OpenArrayApi.Mod` imports it, passes client-declared fixed arrays and selected rows, passes a string to the exported value character-array procedure, observes mutation through an exported `VAR` procedure, and proves the named terminal type retains identity across the interface.

`tests/errors/` must fail with exact diagnostics.

- `OpenArrayBad.Mod` covers scalar actuals, rank mismatches, different terminal basic types, distinct named array and record terminal types, a string passed to a non-CHAR or multidimensional open formal, a string passed to a `VAR` open formal, and wrong arity beside ABI-expanded formals.
- `OpenArrayMutabilityBad.Mod` assigns to a value open formal, to its element, field, and nested selected element; passes it to a `VAR` open formal and to a modified predefined operation through a selection; and passes an imported array or one of its rows to a `VAR` open formal. It also indexes a value open array of pointers, then rejects field mutation, explicit dereference mutation, `NEW`, and `VAR` passing through that pointer. The corresponding value calls remain positive coverage.
- `OpenArrayAssignmentBad.Mod` covers assignment from a fixed source to an open destination, assignment between open formals, an open source assigned to a fixed destination with a different immediate element type, and whole multidimensional open-to-fixed assignment whose immediate element types differ. It keeps string assignment to a value open character array rejected as read-only.
- `OpenArrayLenBad.Mod` uses `LEN` of an open dimension in a constant declaration and a fixed array length, then covers bad fields, too many indices, and non-INTEGER indices through open formals. It distinguishes a dynamic open dimension from a fixed dimension reached through `ARRAY OF Row`, whose `LEN` remains a legal constant.
- `tests/errors/modules/open-array-import-write/` proves an imported fixed array and every selected subarray are valid value open actuals but invalid `VAR` open actuals, while the exported procedure signature itself remains usable.

`tests/failures/` compiles each module, runs it, and compares standard error after a nonzero exit.

- `OpenArrayIndexLow.Mod` indexes an open formal at `-1`, proving even a constant source index uses the runtime actual length.
- `OpenArrayIndexHigh.Mod` passes a shorter fixed array than the procedure's failing index assumes and expects the existing `array index out of bounds` message.
- `OpenArrayInnerIndexHigh.Mod` indexes within the outer bound of a multidimensional open formal but beyond the actual inner length, proving each dimension uses its own hidden value.
- `OpenArrayZeroIndex.Mod` indexes a zero-length actual through an open formal.
- `OpenArrayCopyTooLong.Mod` assigns a longer open source to a shorter fixed destination and expects `array assignment exceeds destination length` before any copy.
- `OpenArrayStringTooLong.Mod` assigns a string whose characters fit exactly but whose appended null does not fit through a writable open character array, expecting the same assignment-length failure.

## Verification

1. Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `git diff --check`.
2. Run every new positive binary directly. Confirm a zero exit status, empty standard error, and byte-for-byte expected standard output.
3. Inspect `build/OpenArrays.ssa`. Confirm each one-dimensional open formal is `l` data followed by one `w` length, value and `VAR` forms have the same machine signature, scalar source arity still counts one formal, and no callee prologue copies array data. Confirm `a, b: ARRAY OF INTEGER` expands independently as `l,w,l,w`, and the two different actual lengths reach the correct formal names.
4. Inspect `build/OpenArrayDimensions.ssa`. Confirm a three-dimensional open formal is one `l` followed by three `w` values in outer-to-inner order. Confirm fixed calls pass declared immediates, all-open forwarding calls pass incoming temporaries, mixed `ARRAY OF Row` forwarding passes its dynamic outer value and `Row`'s fixed inner immediate, and selected-row calls pass only remaining lengths. Confirm the side-effecting selected actual evaluates its index once.
5. Inspect every open index in the multidimensional IL. Confirm the applicable length temporary is passed to `oberon_check_index` before extension or address arithmetic, the outer stride multiplies all remaining dynamic lengths by the fixed terminal size, each selected inner dimension drops one factor, a named fixed-array base uses its complete static size, and fixed indexing still emits the same immediate form as before this slice.
6. Inspect executable `LEN` lowering. Confirm a bare open formal returns its incoming length without a load or check, `LEN(a[i])` checks and computes the selected address before returning the next length, fixed inner `LEN` remains immediate, and no dynamic open `LEN` folds into a constant declaration.
7. Inspect string calls and relations. Confirm each literal object contains one appended null, its passed length includes that null, `LEN` observes the same count, and every comparison passes the open formal's dynamic bound to `oberon_str_cmp` without reading or scanning for a capacity.
8. Inspect open-source assignment. Confirm the source and destination lengths reach `oberon_check_array_copy` before `oberon_copy`, the accepted source length is widened before multiplication, the multiplier is the exact immediate element size, only the source prefix is copied, and zero length reaches the existing zero-byte copy path. Confirm a string-to-open assignment checks characters plus the null and then copies exactly that constant count.
9. Run all six new runtime-failure binaries. Confirm the four indexing cases keep the exact existing bounds message and the two assignment cases produce the exact new message with nonzero status. For the copy failures, initialize and inspect a destination through a temporary diagnostic run to confirm the check occurs before any byte changes, then keep the committed corpus output limited to stable stderr.
10. Compile the cross-module gate and inspect both sides of every exported call. Confirm the declaration and client agree on expanded order and classes, the interface preserves the open rank and terminal named-type identity, and no module-local reconstruction changes the ABI.
11. Confirm read-only propagation through value open formals, nested selections, pointer dereference, forwarding, and imported actuals. Compile the explicit open-array-of-pointers negative gate to prove indexing preserves read-only status before implicit or explicit dereference, `NEW`, and `VAR` substitution. Confirm writable `VAR` views mutate caller storage and aliasing remains observable through a simultaneous value view.
12. Rebuild representative unchanged INTEGER, SET, REAL, fixed-array, record, string, and pointer programs and compare their IL with the Slice 13 results. Generalizing index lengths and byte counts must not alter fixed-only code generation.
13. Search the compiler for every use of fixed array length. Confirm layout and object-size paths accept only fixed arrays, while indexing, `LEN`, character bounds, calls, and open assignment deliberately accept dynamic lengths. Compile malformed and mismatched gates to prove no open type reaches `size`, `align`, `storage`, a scalar load, or an internal panic.
14. Confirm that no newly valid open-array program can reach an unsupported diagnostic, panic, QBE parse error, assembler error, or linker error, and that record extension and procedure types stop at their stable Slice 15 and 16 diagnostics.

## Order of work

1. Parse the formal-type open prefixes, record them in the AST, and update parser shape and unsupported-boundary tests.
2. Add the semantic open-array form, dedicated formal resolution, structural compatibility, display, and no-layout invariants.
3. Expand formal bindings, procedure interfaces, IR parameters, calls, fixed actuals, forwarding, selected subarrays, and string actuals into the address-plus-lengths ABI.
4. Carry fixed and dynamic shape through places, generalize the IR index bound and stride, and update executable indexing and `LEN` for one and several dimensions.
5. Update required-constant `LEN` and the constant designator walks so dynamic open dimensions do not fold while selected fixed dimensions still can.
6. Extend value and `VAR` mutability, open character-array relations, and terminator-inclusive string behavior through the shared place and length representation.
7. Add checked assignment from an open source to a fixed destination, string assignment to a writable open character array, the explicit IR check and dynamic copy, and the runtime failure.
8. Add the positive, negative, runtime-failure, cross-module, ABI, and boundary gates, plus a regression module for every bug found during implementation.
9. Update the architecture document and complete the verification list.
