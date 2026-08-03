# Slice: Records and structured parameters

## Context

After Slice 11 the compiler supports every basic type of Report 6.1, fixed arrays with checked indexing and whole-array assignment, strings and their compatibility rules, and a module system whose interfaces carry exported types with one identity across the build. Two of its types are one byte wide, so layout code already answers real alignment questions instead of a constant four. Every aggregate so far is homogeneous: an array's elements all share one type, one size, and one stride, and no address computation has ever needed anything but a scaled index.

This slice adds record types without extension: named and inline records, field declarations with export marks, field selection, nested records, arrays of records, records containing arrays, and whole-record assignment. It also extends procedures to structured parameters, so a fixed array or a record can be an actual parameter for the first time. It is the first slice with heterogeneous layout and padding, the first with a field-offset address computation, and the first in which a parameter's storage lives in the caller's frame.

Report sections implemented: section 6.3 for record types, field lists, and public and private fields, excluding the `(BaseType)` extension form; section 8.1 for the field selector in designators; section 9.1 for record assignment and for the rule that a structured value parameter permits no assignment to it or to its elements; section 9.2 for parameter substitution; section 10.1 for structured formal parameters, the named-type restriction of `FormalType`, and the exclusion of array and record result types; and section 11 for record types and record-typed variables crossing a module boundary.

Implementation starts from a reviewed Slice 11 with a green common gate. Slice 11's one-byte types and its single assignment-compatibility function, together with Slice 10's byte-copy machinery, are the baseline this slice builds on.

## Language and representation decisions

### Layout is declaration order with per-type alignment

A record's fields are laid out in declaration order. Each field is placed at the next offset that is a multiple of its type's alignment. The record's own alignment is the largest alignment among its fields, and its total size is the end of its last field rounded up to that alignment, so an array of records strides correctly. One function computes all of it — offsets, alignment, and size — when the record constructor is resolved, and checks the total against the existing target object-size limit. The offsets it assigns are the offsets everything downstream uses; nothing recomputes layout later.

The alignment of every type the compiler has is either one or four, so the rule's first real exercise is a CHAR or BYTE field followed by a four-byte one. `RECORD ch: CHAR; count: INTEGER END`, the element type of the Report's own section 7 example, is ch at offset 0, three bytes of padding, count at offset 4, size 8, alignment 4. Padding bytes have no value a program can observe: the language has no way to read them, and a whole-record copy moves them along with everything else, which is harmless.

Declined: Project Oberon's layout rule, which aligns every field whose size exceeds one byte to a four-byte boundary and rounds every record and array size up to a multiple of four. Under that rule `ARRAY 3 OF CHAR` occupies four bytes; under Slice 10's rule, which this compiler keeps, it occupies exactly three. The per-type alignment rule is the one the existing array layout already implies, and it is the layout a C compiler would give the same struct, which is worth having when the runtime is C.

An empty record — `RECORD END` is legal, since the field list sequence is optional — has size zero and alignment one. Assigning one resolves both designators and copies zero bytes, exactly as a zero-length array assignment does. A zero-length array as a field occupies no bytes, so a record containing one gains no storage for any element. The field's alignment still participates in layout like any other field's, so it can move a later field or the record's total size the way any aligned field can; what it can never do is reserve element storage.

### A record constructor is one type, and it remembers its name

Record identity follows Slice 10's array rule: each `RECORD` constructor written in the source creates one type, two types are the same only when they share it, and a type declaration whose right side is an existing name is an alias sharing the existing identity. Two records with identical field lists written separately are different types, and assignment between them is an error.

Arrays print their structure in diagnostics, but a record's structure is too large to print. A record descriptor therefore remembers the name of the type declaration whose right side its constructor was, and diagnostics print that name. A record constructor written inline in a variable declaration or a field list has no name and prints as `RECORD`. When an assignment or argument diagnostic would show two identical spellings for two different record types, the existing "these are different array types" hint gains a record twin.

### Fields carry export marks, and visibility is decided by the defining module

A field identifier in a field list is an `identdef`, so it may carry an export mark. The record descriptor stores the mark with the field, and it also stores the name of the module that declared the constructor. Field lookup then has one rule: an unmarked field is visible only inside its defining module. Inside the defining module every field of every record is visible, marks included or not. In any other module, only marked fields exist; an unmarked one gets the same diagnostic as a field that was never declared, so a private field is indistinguishable from an absent one. All three reference compilers behave this way, and it avoids leaking private names into other modules' error messages.

This rule needs no reachability analysis. A client that imports a record type, a client that declares its own variable of that type, and a client that reads an exported variable whose record type is private all go through the same lookup against the same descriptor, and the descriptor knows where it was born. An exported variable of a private record type is therefore readable and its marked fields are selectable, while the type name itself stays unavailable for declaring anything — the same shape Slice 10 gave exported variables of private array types.

A field export mark is legal only when the record type is declared at module scope, including a record nested inside another module-scope declaration. A mark on a field of a record declared inside a procedure is a diagnostic, following Project Oberon's "remove asterisk" rule; a procedure-local type can never be visible outside the module, so the mark could never mean anything.

The scope of field names is the record itself, as Report 6.3 states, so a field may share its name with any module or local declaration without conflict. Two fields of one record with the same name are a diagnostic.

### Structured value parameters are read-only references

Report 9.2 says a value parameter's actual is evaluated prior to activation and its value assigned to the formal. Report 10.1 then qualifies it: "if a value parameter is of a basic type, it represents a local variable to which the value of the actual expression is initially assigned" — basic types only. For structured value parameters the Report instead adds the 9.1 rule that no assignment to the parameter or to its elements is permitted. That pairing exists to license an implementation that passes the address and never copies, and all three reference compilers read it that way: Project Oberon reclassifies a structured value parameter as a reference parameter with a read-only mark, OBNC emits a `const` pointer, and oberonc passes the JVM object reference. This compiler does the same. A structured value parameter receives the address of its actual, makes a local copy of nothing, and is read-only in its entirety — the parameter, its elements, its fields, and their elements and fields, all the way down.

The consequence is that aliasing is observable. If one variable is passed both as a structured value parameter and as a `VAR` parameter of the same call, an assignment through the `VAR` parameter is visible through the value parameter immediately. That is the behavior of every reference compiler, and this slice pins it with a corpus test rather than leaving it to chance.

Declined: copying the actual into the callee's frame, which would make a structured value parameter a true local snapshot. It reads more like the word "value", but it contradicts all three references, it makes every call with a large array actual cost a copy the Report's read-only rule exists to avoid, and the Report's own careful restriction of the local-variable sentence to basic types tells against it.

Read-only-ness propagates the way the imported-variable rule already does: the place a designator resolves to carries the flag from its base symbol through every selector. A structured value parameter's fields and elements reject assignment, rejection covers `INC`, `DEC`, `INCL`, `EXCL`, `PACK`, and `UNPK` targets, and a read-only designator cannot be a `VAR` actual. Passing a read-only designator onward as another structured *value* actual is fine — that is a read.

### Actual parameters for structured formals are designators of identical type

A structured formal — value or `VAR` — takes the address of its actual, so the actual must be a designator denoting a variable, and its type must be identical to the formal's. For `VAR` parameters, identity is what Report 10.1 demands; its record exception, where the formal may be a base type of the actual, becomes meaningful only when extension arrives in Slice 15. For structured value parameters, identity is what remains of assignment compatibility once the actual must be addressable: no expression the language can currently form yields a record or array value that is not a designator.

A string actual for a fixed character-array formal is a diagnostic, and the rejection is permanent. The Report is genuinely ambiguous here. Section 9.2 describes value-parameter passing as assigning the actual's value to the formal, and under that reading section 9.1's string exception would apply; but section 10.1 restricts the formal-is-a-local-variable sentence to basic types, and under this compiler's reference semantics no assignment to a structured formal ever happens, so there is nothing for the exception to attach to. The references split. Project Oberon and oberonc both accept a string actual only when the formal is an open array of CHAR and reject it for a fixed one; OBNC routes value actuals through assignment compatibility and accepts it. This compiler follows the majority and the reference semantics. The cost of accepting it would not be a runtime copy — the literal's data object could be padded to the formal's declared length and its address passed — but it would be a second way for a formal to bind, used only by this one case, and Slice 14's open arrays of CHAR are where the Report unambiguously wants strings to meet parameters.

The diagnostic fires for a string of any length, including one. Slice 11's contexts that accept a string examine the expression before lowering it, and the structured-actual check is such a context: a string actual against a structured formal is reported as a string where a variable is required, not as a CHAR mismatch left over from the single-character rule. A single-character string actual for a CHAR formal keeps working through assignment compatibility, as it did in Slice 11.

The formal parameter grammar needs no new syntax: `FormalType = {ARRAY OF} qualident`, so a structured formal is always written as a named type, and the parser already parses that. What changes is that semantic analysis stops rejecting a formal whose named type resolves to an array, and starts accepting records. Open-array formals — the `ARRAY OF` prefix — remain Slice 14 and keep their parser diagnostic.

Result types gain the record rejection alongside the existing array one. Report 10.1: the result type of a procedure can be neither a record nor an array. Both diagnostics are permanent.

### Record assignment copies, and records have no equality

Assignment between two designators of the same record type copies the record's complete byte representation, padding included, through the existing byte-copy runtime call. The destination is resolved first and the source second, each exactly once, and self-assignment is legal because the copy is a `memmove`. Report 9.1's record rule — the source must be an extension of the destination's type — reduces to identity while extension does not exist; Slice 15 widens it.

The relations `=` and `#` do not apply to records: Report 8.2.4 lists BOOLEAN, SET, pointer, and procedure types for equality, and records are absent. A record operand in any relation is a diagnostic, as it is in arithmetic, in a condition, in a `CASE` selector, and in every other value context. All three reference compilers reject record comparison.

### No extension, no forward references

The `RECORD (Base)` extension form parses to a stable "not yet supported" diagnostic and is otherwise untouched until Slice 15. Record declarations resolve their field types at the point the constructor is read, so every field type must already be declared; the Report's one forward reference — a pointer type naming a record declared later in the same scope — arrives with pointers in Slice 13, and until then no legal source can mention a type before its declaration. A record cannot contain itself by value under any reading, so nothing recursive is expressible or needs detecting: `TYPE T = RECORD f: T END` names an undeclared identifier, because a type declaration resolves its right side before declaring its name. The layout function silently depends on that no-cycle guarantee, and Slice 13 will change name-visibility timing for pointer bases, so this slice pins the self-referential field's diagnostic with a test now.

## Semantic types and module interfaces

The semantic type gains a record case holding a shared descriptor, exactly parallel to the array case: the descriptor owns its field list — each field a name, a type, a computed offset, and an export mark — plus the record's size, alignment, declared name if any, and defining module. Type identity is descriptor identity. The layout function that fills offsets, alignment, and size runs once at construction and applies the object-size limit as array construction already does.

The scalar helper answers nothing for a record, so no record can reach a load, a store, or a scalar argument by accident; every context that means "a value" already diagnoses what it cannot lower. The type display prints the declared name or `RECORD`.

Field selection joins index selection in the designator walk. The walk resolves a field by name against the descriptor, applies the visibility rule, adds the field's offset to the running address, carries the base's read-only flag through unchanged, and continues with the field's type. All three selector walks — the runtime place resolution, the constant designator type walk, and the constant designator check — handle the field selector identically, so `LEN` of an array field folds in constant contexts exactly as `LEN` of an array variable does. The existing "record field selection is not yet supported" diagnostic is deleted, and its other job — a selector applied to something that is not a variable at all — gets its own message.

Module interfaces need no new member kind. An exported record type travels as a type member whose descriptor is shared, so cloning an interface preserves identity across every client and every re-export, and the visibility rule works because the descriptor carries its defining module wherever it goes. Exported variables and procedures with record types travel through the existing variable and procedure members.

The assignment-compatibility function gains nothing: its whole-copy outcome, written for identical array types in Slice 11, states the rule by asking whether the two types are identical structured types, and records satisfy it the day the type exists. The string-into-character-array outcome likewise works when the destination is a character-array field, because the outcome is decided by types and a field's place carries its type.

## IR and QBE lowering

### Storage is a size and an alignment

The IR's storage description gains a record form carrying just a size and an alignment. Globals are zero-filled reservations and slots are frame reservations, and neither needs to know where fields sit — field positions live entirely in the address computations the front end emits. A record global is emitted as `data` with its type's alignment and a zero fill of its exact size; a record local reserves its exact size in the activation record. Frame and module storage accounting already round a running offset up to each object's alignment, and this slice is the first to exercise that rounding with mixed alignments, including the copy of the same rounding in the driver's static-data check — the two must stay in step.

### Field addressing is one instruction

The IR gains a field instruction: destination, base address, byte offset. QBE lowering is a single `add` of the constant offset to the base pointer. It is emitted for every field selection, including offset zero — there is no optimization pass, and one literal path is the same choice Slice 10 made for constant indices. Nested selections chain: `a[i].f.g[j]` is an index, an add, an add, an index, each step feeding the next base, each index still carrying its length and check. The existing index instruction is untouched; a record field of array type simply supplies the array's base address to it.

### Structured parameters are addresses in the existing call machinery

A structured parameter, value or `VAR`, is a reference parameter in the IR: the formal binds to an incoming pointer temporary, and the call site passes an address argument. Both use the QBE pointer class, which the calling convention already has for scalar `VAR` parameters, so no new argument form, no QBE aggregate types, and no prologue copy exist anywhere. The value-versus-`VAR` distinction is entirely semantic — it decides read-only-ness and what the actual may be — and is gone by the time the IR sees the call.

The QBE slot allocator maps alignment to its allocation instruction; every record's alignment is one or four today, and slots stay `alloc4` exactly as Slice 11 chose for byte-wide locals.

## Procedures and predefined operations

Records work in every legal role this slice can reach: module variables, procedure locals, fields of other records, array elements, value parameters, and `VAR` parameters. Fixed arrays gain the two parameter roles. Function results reject both, permanently. Record constants do not exist because no constant expression can denote a record.

Designators reaching through the new selector feed everything that takes a designator: `INC(w[k].count)` — the Report's own section 9.2 example — resolves the element, the field, and the increment through the ordinary paths; `LEN` of an array field answers its declared length; a character-array field takes a string assignment and joins character-array comparisons. Predefined operations that require scalars diagnose record arguments through the existing argument checks.

## What remains unsupported after this slice

- Record extension, the `RECORD (Base)` form, extension-to-base assignment and parameter passing, type tests, type guards, and the record and pointer forms of `CASE` remain in Slice 15.
- Pointers, `NIL`, `NEW`, dereference selectors, and the forward reference to a record type remain in Slice 13.
- Open array formals remain in Slice 14, so a string still cannot be passed to any procedure, and `Out.String` still does not exist. Passing a string to a fixed character-array formal is rejected in this slice and stays rejected; strings meet parameters through open arrays of CHAR.
- Procedure types remain in Slice 16.
- Array and record result types and record equality are excluded by the Report, not deferred.
- The optional `SYSTEM` module remains outside the core roadmap, so padding and layout stay unobservable from source.

The scalar representations, string rules, array layout, checked indexing, module initialization, and scalar calling convention from prior slices remain unchanged.

## Changes by file

### src/ast.rs

Add a record type expression holding its field lists: each entry a list of `identdef` names, a type expression, and positions. The existing `identdef` node already carries the export mark. Nothing changes for designators — the field selector node has existed since module qualifiers needed it.

### src/parser.rs

Parse the record type production: optional field list sequence, fields as `identdef` lists with a type, semicolons between field lists and none before `END`, per the grammar file. A `(` after `RECORD` is the extension form and produces "not yet supported: record extension". Replace the parser test that expects `RECORD` itself to be unsupported with tests for a flat record, a nested inline record, an array-of-record type expression, marked and unmarked fields, an empty record, and the extension diagnostic.

### src/sema.rs

Add the record type case and its descriptor, the layout function with the object-size check, the name stamping in type declarations, and the record display. One resolution-order detail: a type declaration resolves its right side before declaring its name, so the declared name is not in scope when the descriptor is built — thread the name into type resolution for the one case where the right side is a record constructor, rather than mutating the shared descriptor afterwards. Resolve record constructors in type declarations, variable declarations, and field types; diagnose duplicate fields and marks on procedure-local record fields.

Add the field selector to all three designator walks with the visibility rule, offset accumulation, and read-only propagation. Split the old selector diagnostic into "no such field" — shared by absent and private fields — and a message for selecting from a non-variable.

Delete the array-parameter rejection. Bind structured value formals as read-only reference parameters and structured `VAR` formals as the reference parameters scalars already have. Check structured actuals as designators of identical type, routing value and `VAR` structured actuals through place resolution rather than expression lowering, and diagnose a string actual against a structured formal. Reject record result types permanently.

Extend the whole-copy outcome of assignment compatibility to identical record types, emit the byte copy with the record's size, and add the record twin of the identical-spelling hint. Reject records in relations and every other value context through the existing scalar gate.

### src/ir.rs

Add the record storage form carrying size and alignment, and the field instruction carrying destination, base, and offset. Nothing else: calls, parameters, and the byte copy already have every form structured parameters and record assignment need.

### src/qbe.rs

Emit the field instruction as one pointer `add`. Emit record globals with their alignment and zero size fill, and record slots by size under the existing allocation rule. No changes to calls or parameter classes — references already pass as pointers.

### runtime/oberon.c

Nothing. Record copies reuse `oberon_copy`, and no new runtime check exists in this slice.

### docs/dev/architecture.md

Add a records section covering the layout rule with a padding example, record identity and the declared-name display, field export marks and the defining-module visibility rule, the structured parameter convention with the aliasing consequence, and record assignment. Note in the parameter discussion that a "value" parameter of structured type is an address with a read-only rule, and why.

## New corpus modules

`tests/corpus/` compiles each module, runs it, and compares standard output.

- `Records.Mod` covers record module variables, locals, nested records, and every basic type as a field. It reads and writes fields directly and through nesting, copies whole records, and proves the copy is a copy by mutating the source afterwards and printing the untouched destination. It includes an empty record that is assigned, and the record `RECORD n: INTEGER; z: ARRAY 0 OF INTEGER END`, whose `n` field is read and written and whose `z` field's `LEN` is zero — the zero-length field shares the record's alignment, so the record's size is exactly its other field's, which verification checks in the emitted data.
- `RecordLayout.Mod` declares records that force padding — a CHAR before an INTEGER, a BYTE array between REALs — writes distinctive values into every field, copies the records whole, and prints every field of the destination, so a wrong offset or a wrong size shows up as wrong output.
- `RecordTable.Mod` is the Report's section 7 shape: `ARRAY 16 OF RECORD ch: CHAR; count: INTEGER END`. It assigns fields through indexed elements, runs `INC(w[k].count)`, assigns one element to another, compares character-array fields, and assigns a string to a character-array field and scans for its terminator. It also copies one whole array of records to another — exercising the rounded record stride — and proves that copy is a copy by mutating the source afterwards.
- `RecordParams.Mod` passes records and fixed arrays as value and `VAR` parameters, nests calls, reads fields and elements of value parameters, mutates through `VAR` parameters, and returns scalars computed from structured parameters. A recursive procedure takes a record `VAR` parameter.
- `ParamAlias.Mod` pins the aliasing decision: a procedure takes the same record as a value parameter and a `VAR` parameter, assigns through the `VAR` parameter, and prints the value parameter's field before and after, showing the mutation is visible. A second procedure does the same with an array.

`tests/corpus/modules/record-api/` supplies the cross-module coverage.

- `RecordTypes.Mod` exports a record type with marked and unmarked fields, an exported variable of that type initialized in its body, an exported variable of a private record type with a marked field, and procedures taking the exported type by value and by `VAR`.
- `RecordApi.Mod` imports it, declares variables of the exported type, assigns between its own and the imported variable, selects marked fields of both exported variables, and passes records to the imported procedures, proving type identity across the boundary.

`tests/errors/` must fail with exact diagnostics.

- `RecordBad.Mod` covers record equality and ordering, a record in arithmetic and in a condition, an unknown field, a duplicate field, a field selector on an INTEGER, assignment between two identically written record types with the hint, a record result type, and the extension form's diagnostic.
- `RecordParamBad.Mod` covers assignment to a structured value parameter, to its field, and to an element of an array field; a structured value parameter as a `VAR` actual; `INC` of a value parameter's field; a `VAR` record actual of a different record type; a record *value* actual of a different record type; a fixed-array `VAR` actual of a different array type with the same shape; and string actuals for a fixed character-array formal, one longer than the array and one of a single character.
- `RecordExportBad.Mod` covers an export mark on a field of a procedure-local record.
- `RecordRecursive.Mod` pins `TYPE T = RECORD f: T END` to its undeclared-identifier diagnostic, so Slice 13's change to name-visibility timing cannot silently let a self-referential record reach layout.
- `tests/errors/modules/record-private/` has a client selecting an unmarked field of an imported record type and of an exported variable of a private type, both getting the no-such-field diagnostic.
- `tests/errors/modules/record-import-write/` proves an imported record variable, its fields, and elements of its array fields all reject assignment, and that an imported record cannot be a `VAR` actual.

The existing `tests/errors/ArrayParamBad.Mod` loses its reason to exist when array parameters become legal; it is replaced by positive coverage in `RecordParams.Mod` and the negative coverage above.

`tests/failures/` compiles each module, runs it, and compares standard error after a nonzero exit.

- `FieldIndexBounds.Mod` indexes an array field of a record out of bounds through a structured `VAR` parameter, proving the existing check runs on the new addressing path. No new dynamic check exists in this slice, so no new failure message does either.

## Verification

1. Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `git diff --check`.
2. Run every new positive binary directly. Confirm a zero exit status, empty standard error, and byte-for-byte expected standard output.
3. Inspect `build/RecordLayout.ssa`. Confirm each field selection is one `add` with the offset the layout rule predicts, that padded offsets appear where CHAR and BYTE fields precede four-byte ones, that record globals carry the computed alignment and exact size, and that whole-record copies pass the full padded size to `oberon_copy`.
4. Inspect `build/RecordParams.ssa`. Confirm structured value and `VAR` parameters both arrive as one pointer-class argument, that no copy is emitted in any callee prologue, and that a scalar value parameter still copies into its slot.
5. Confirm `ParamAlias.Mod`'s output shows mutation through the `VAR` parameter visible through the value parameter, pinning the by-reference decision.
6. Inspect frame layouts in a procedure mixing CHAR, record, and INTEGER locals. Confirm slot reservation rounds offsets to each type's alignment and that the module and driver storage accounting agree.
7. Confirm the zero-length-array record from `Records.Mod` reserves no element storage: its emitted global and its copy size equal the size of its `n` field alone, since the zero-length field shares the record's alignment and so adds no padding either.
8. Compile the cross-module gate and confirm assignment between the client's and the exporter's variables of one exported type works, that private fields are absent from the client with the same message as unknown fields, and that no descriptor is duplicated — the type is identical on both sides.
9. Rebuild unchanged programs from prior slices and compare their IL with the pre-slice results. Adding records must not alter existing code generation.
10. Confirm that no newly valid record program can reach an unsupported diagnostic, a panic, a QBE parse error, an assembler error, or a linker error, and that extension, pointers, and open arrays stop at their stable Slice 13, 14, and 15 diagnostics.

## Order of work

1. Parse record types: field lists, marks, the empty record, and the extension diagnostic, with parser tests.
2. Add the record descriptor, the layout function, name stamping, display, and the object-size check.
3. Add the field selector to the three designator walks with visibility, offsets, and read-only propagation, and split the old selector diagnostic.
4. Add the record storage form and the field instruction to the IR and QBE, and emit record globals and slots.
5. Extend assignment compatibility's whole-copy outcome to records and emit record copies.
6. Replace the array-parameter rejection with structured value and `VAR` parameters: binding, actual checking, and the string-actual diagnostic.
7. Add field export marks, the defining-module visibility rule, and the procedure-local mark diagnostic.
8. Add the positive, negative, cross-module, and failure gates, and a regression module for every bug found during implementation.
9. Update the architecture document and complete the verification list.
