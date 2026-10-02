# Slice: Pointers, NIL, and garbage-collected allocation

## Context

After Slice 12 the compiler has fixed arrays and records with explicit identity, exact size and alignment, field visibility across modules, whole-value copies, and structured parameters that travel by address. Every value that has crossed a procedure boundary so far has used a QBE word, single, or reference to caller-owned storage. No source value has itself been an address, no type graph has been recursive, and every selector has derived an address from storage that already existed.

This slice adds pointer types bound to records, including the Report's forward reference to a record in the same scope. It adds `NIL`, pointer assignment and equality, pointer parameters and results, `NEW`, explicit dereference with `^`, and the implicit dereference performed by field selection on a pointer. It is the first slice with an eight-byte scalar, recursive semantic type graphs, heap storage, values that use QBE's long class, and runtime checks that turn a loaded value into an address.

Report sections implemented: section 3 for `NIL`, `POINTER`, and `^` in the vocabulary; section 4 for predefined `NEW`; section 6 and section 6.4 for pointer types bound to records, the permitted same-scope forward reference, `NIL`, and allocation, excluding the inherited extension relation whose nontrivial cases require Slice 15; section 7 for pointer variables; section 8.1 for explicit dereference and implicit dereference during field selection; section 8.2 for `NIL` as a factor; section 8.2.4 for pointer equality and inequality; section 9.1 for pointer assignment and the `NIL` exception; section 9.2 for pointer actual parameters; section 10.1 for pointer formals and results; section 10.2 for `NEW`; and section 11 for exported pointer types, variables, and procedures.

Implementation starts from the current Slice 12 working tree. Its common gate is green: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` all pass, including the record and structured-parameter corpus.

## Language and representation decisions

### A pointer is an eight-byte scalar with its own type identity

The supported QBE target defaults to `amd64_sysv`, the native C compiler reports an eight-byte pointer, and QBE represents an address in its long class. A pointer therefore occupies eight bytes, has alignment eight, loads with `loadl`, stores with `storel`, and travels in QBE's `l` class as a value parameter or result. A pointer variable is still storage: its address is distinct from the pointer value loaded from it. This distinction matters for a `VAR` pointer parameter, which passes the address of the pointer variable, and for dereference, which loads the pointer value and then treats that checked value as the address of a record.

Each `POINTER` constructor creates one pointer type. Aliases and imported interfaces share its descriptor, while two separately written constructors are different types even when they name the same base record. This keeps the constructor-identity rule already used for arrays and records and makes Report 10.1's identical-type requirement for `VAR` parameters literal.

Ordinary pointer value compatibility is broader than identity. Two pointer types bound to the identical record type are assignment-compatible and may be compared for equality, even when their pointer constructors differ. Project Oberon and OBNC derive this from the pointer extension relation, and oberonc calls pointer types with equal bases equal. With no record extension yet, identical bases are the only non-`NIL` compatible case. Slice 15 widens this helper to the inherited extension relation rather than replacing the pointer representation.

A pointer `VAR` formal requires the actual's identical pointer type, not merely an identical base record. Project Oberon and OBNC enforce that Report 10.1 identity rule; oberonc instead accepts a separately constructed pointer with the same base, so the Report and the two stricter references determine this slice's behavior. A pointer value formal, assignment, or `RETURN` clause uses ordinary pointer value compatibility and therefore accepts another pointer type with the same base. Diagnostics for two differently constructed pointer types that print alike include the same identity hint arrays and records already receive.

Declined: representing a pointer as an `ir::Addr`. An IR address identifies storage that an instruction may load or store; a pointer value can be `NIL`, can travel through a temporary, and must be checked before it becomes an address. Collapsing the two would make it easy to omit a load, pass the address of a pointer variable where its value is required, or dereference without a check.

### Pointer bases are records, including inline records

`POINTER TO type` parses the full `type` production, as the Report's grammar says. Semantic analysis accepts the result only when it is a record type. A named record, an alias of one, an imported exported record, and an inline `RECORD ... END` are valid bases. An array, basic type, pointer, or later procedure type is not.

A pointer descriptor carries its base record. A known base is installed immediately. While semantic analysis is resolving the right sides of declarations in a `TYPE` section, a bare unqualified base name that is not yet declared creates a pending reference, because that is the one context in which Report 6.4 permits a forward name. A pointer constructor in a variable declaration or any later declaration receives the ordinary undeclared-name diagnostic instead; the grammar cannot place a type declaration after it. The pending reference records the pointer descriptor, target name, and source position. Qualified names are never pending: an imported member either exists already or receives the ordinary module-member diagnostic.

Normal lexical lookup happens before a base becomes pending. If an outer scope already declares the name, the pointer binds that visible record; a declaration of the same name later in the current scope does not retroactively retarget it. Pending bases are resolved after the current `TYPE` section has been processed and only against the same scope. The rule works independently in the module scope and in each procedure scope. It supports `P = POINTER TO R; R = RECORD ... END`, a record field written as `next: POINTER TO R` inside the declaration of `R`, mutually recursive records connected only by pointers, and several pointer types waiting for one later record. It does not make ordinary field types forward-referenceable, does not search a nested procedure for a module-level target, and does not let a pending base resolve to a non-record type.

If an unresolved base remains missing or resolves to a non-record, its pointer descriptor is marked invalid after reporting the source diagnostic. Later declarations and statement lowering may still mention that pointer while diagnostics are being collected, so base lookup returns no record and skips dependent lowering instead of panicking or substituting an invented valid type.

Declined: predeclaring every name in a `TYPE` section. The Report grants a forward reference only to the record base named by a pointer. Making every type name visible early would silently accept invalid forward references in arrays, record fields, and aliases and would change the textual-scope rule that all earlier slices enforce.

### Recursive type graphs are deliberate and bounded

A record may contain a pointer to itself or to another record that eventually points back. The semantic graph therefore becomes recursive: a shared record descriptor owns its field types, and a shared pointer descriptor owns its resolved base record. The pointer descriptor uses an explicit pending, resolved, or invalid base state so a forward reference is filled once and every use can distinguish valid source from earlier diagnostic recovery.

These shared descriptors can form an `Rc` cycle. That cycle lasts only for the compiler process and is bounded by the source type graph, so this slice accepts it rather than replacing the existing shared descriptors with a type arena. Debug formatting for pointer and record descriptors must print names and shallow properties rather than recursively walking bases and fields, so tracing or an invariant failure cannot recurse forever.

### `NIL` is a polymorphic constant represented by zero

`NIL` gains an AST expression, a semantic constant value, and a semantic `NIL` pseudo-type. Its IR value is zero in the pointer class. A constant declaration may bind it, including an exported constant, so `CONST None* = NIL` behaves like the literal in every client.

`NIL` is assignment-compatible with every pointer type. It is accepted as a pointer value actual and as a pointer function result, but never as a `VAR` actual because it has no storage. Equality and inequality accept a pointer with `NIL` in either order, two compatible pointer values, and `NIL` with `NIL`. Pointer ordering remains illegal. `NIL = NIL` and `NIL # NIL` fold, while a comparison involving a pointer variable is emitted with QBE's long equality operation. Project Oberon and oberonc accept the all-`NIL` form; OBNC requires one operand to have a pointer or procedure type, so the Report's statement that pointer relations also apply to `NIL` and the two accepting references determine this slice's behavior.

The pseudo-type has no size, storage form, declared type name, or general arithmetic operations. A use in arithmetic, a condition, `CASE`, `LEN`, indexing, or another scalar operation receives a source diagnostic rather than being coerced to an integer zero.

Module pointer variables use the compiler's existing zero-filled static storage and therefore start as `NIL`. Procedure-local pointer variables remain uninitialized until assigned, like the compiler's other locals. This slice does not promise initial values for fields of a newly allocated record; correct programs assign a field before reading it unless another language rule has initialized it.

### Dereference is one checked transition from a pointer value to record storage

The AST gains an explicit dereference selector for `^`. The semantic place walk applies it only to a pointer place: it loads the pointer value exactly once, emits a nil check, and returns a place whose address is that checked value, whose type is the pointer's base record, and whose read-only flag is inherited from the pointer place.

Field selection reuses that operation. If the current place is a pointer, `p.f` first performs the same checked dereference and then applies the existing record-field lookup and fixed-offset instruction; if it is already a record, field selection behaves exactly as it did in Slice 12. `p^.f` therefore checks once, while `p.next.key` checks `p` before selecting `next` and then checks the loaded `next` before selecting `key`. Explicit and implicit forms cannot diverge because both call one semantic dereference helper.

The IR gains a nil-check instruction carrying the pointer value. QBE lowering calls `oberon_check_nil` before the checked temporary is used as an address. The runtime helper prints `nil pointer dereference` and exits when its argument is null. A check is emitted for every executed dereference, even when data flow could prove a preceding assignment was `NEW` or `NIL`; there is no optimization pass, and the literal invariant is easier to inspect.

Read-only status continues through a dereference. An imported pointer variable, a pointer field reached through an imported structured variable, or a pointer field of a structured value parameter cannot be used to mutate the referenced record or passed to `NEW` or a `VAR` formal. Project Oberon and OBNC both preserve the base designator's read-only status across pointer selectors, and this choice keeps the existing rule that all selectors of a read-only base remain read-only.

Short-circuit evaluation is unchanged and is the source mechanism for guarding a dereference. In `(p # NIL) & (p.key = k)`, the nil check for `p.key` runs only after the left operand is true. The gate pins this interaction because an eager dereference would turn valid list traversal into a runtime failure.

### Fixed `LEN` may fold through a pointer selector

The constant-expression type walk learns explicit dereference and implicit pointer dereference so it can diagnose selector and field errors accurately. For fixed arrays, the Report defines `LEN` from the declared array type, and Project Oberon, OBNC, and oberonc all accept `LEN(p.row)` and `LEN(p^.row)` in required constant contexts. They also accept a variable index inside the designator when the selected result has fixed array type. The folding walk therefore follows selector types without loading pointers or evaluating indices and returns the fixed length. Every index expression must still have type INTEGER, and an index that is itself constant is checked against the fixed bound. This is a narrow property of fixed `LEN`, not a rule that pointer field values or other expressions reached through a pointer are constant.

An executable `LEN(p.row)` or `LEN(p^.row)` remains an ordinary expression. It resolves the actual designator, loads and checks the pointer, and preserves every index evaluation and bounds check before returning the fixed length, as required by the Slice 10 executable-`LEN` rule. A constant declaration performs no runtime evaluation, so its type-only fold cannot dereference a nil pointer or execute an index expression. Slice 10 currently requires constant selectors in this path; this slice corrects that restriction by making required-constant fixed `LEN` type-only for direct array, record, and pointer designators alike. The gate covers both paths explicitly rather than letting ordinary expression lowering erase checks or required-constant folding invent runtime work.

### `NEW` allocates the exact record payload through one of two wrappers

`NEW` is installed in the universe scope as a predefined proper procedure. It takes exactly one writable designator of pointer type. The designator may be a pointer variable, field, or array element, and all selectors are evaluated once before allocation. The existing variable-actual and place machinery resolves the address and read-only status; `NEW` then checks that the resulting type is a pointer instead of supplying the one fixed expected type used by `PACK` or `INCL`. An imported variable, a read-only structured value parameter, a non-designator, and a non-pointer therefore follow the established writable-actual diagnostics.

Lowering obtains the resolved base record, emits one allocation instruction with the record's exact byte size and allocation kind, and stores the returned pointer into the target. The allocation instruction lowers to `oberon_alloc` for a record that may contain pointers and to `oberon_alloc_atomic` otherwise. The wrappers already return the allocator's pointer unchanged, so allocation failure naturally stores `NIL` as Report 6.4 requires. No compiler code calls BDWGC directly.

Resolving a selected target can leave only an address into a heap object live across the allocator call. For example, `NEW(p.slots[f()])` computes the array-element address before allocating and stores through it afterwards. `oberon_init` therefore enables BDWGC's arbitrary interior-pointer recognition before `GC_INIT`, rather than relying on how the installed collector was built. The live selected address is then a valid conservative root while allocation may collect its containing object. This also preserves the once-only and before-allocation evaluation order for target selectors.

The record descriptor gains a `contains_pointers` property computed with its layout. A pointer field is enough to make it true without following the pointer's base, which terminates recursive type graphs. A positive-length array inherits the property of its element, while a zero-length array contains no storage and therefore no pointer even when its element type is a pointer. A nested record uses its already computed property. An empty record and a record composed only of basic values, pointer-free arrays, and pointer-free records are atomic. Any pointer occupying storage at any nesting depth selects the scanned allocator.

Declined: sending every allocation through `oberon_alloc`. Conservative scanning would remain correct, but it would discard the roadmap's required atomic path and make pointer-free byte buffers retain unrelated heap objects when their bit patterns happen to resemble addresses.

Declined: selecting `oberon_alloc_atomic` from only the immediate fields. A pointer inside an array or nested record is still a live heap reference. The allocation kind is chosen from the complete source layout, not from the outer record's superficial shape.

Declined: adding a heap header or a runtime type descriptor. Slice 13 needs only the record payload and its compile-time pointer-containment property. Slice 15 earns runtime type identity when extension, type tests, guards, and type cases need it.

### Generated pointer storage is visible to the conservative collector

Pointer module variables are aligned data objects, pointer locals and value parameters are stack slots, pointer `VAR` parameters name caller storage, and pointer temporaries use QBE's long class. BDWGC scans the executable's static data, stack, spills, and saved registers conservatively, so these ordinary native locations are the roots. A pointer-containing heap object is allocated through the scanned wrapper, which lets a field retain another object. A pointer-free object is atomic and deliberately contributes no roots.

The collector gate retains a list only through pointers held by generated module and procedure storage, allocates hundreds of thousands of temporary objects to force collections, and then traverses the retained list. It also performs `NEW` through a selected field and selected array element of a heap record while allocation churn can collect, proving the selected interior address remains a root across the allocator call. A separate verification run enables the installed collector's `GC_PRINT_STATS=1` diagnostics and confirms that collection actually occurred. The ordinary corpus run leaves that environment variable unset so standard error remains empty.

## Semantic types and module interfaces

The semantic type gains a pointer case holding a shared descriptor and a `NIL` pseudo-type. The pointer descriptor records the source spelling used for diagnostics and its pending, resolved, or invalid base state. Type identity is descriptor identity. A helper answers the resolved base record without exposing the mutable forward-fixup state to assignment, selector, allocation, or interface code.

Pointer value compatibility asks whether both resolved base records have identical record descriptors. The assignment helper uses that rule for pointer-to-pointer values and accepts `NIL` for any pointer target. The relation checker uses the symmetric form for equality and inequality. The exact-type check used by `VAR` parameters remains descriptor identity.

Pointer types are scalar, not structured. A pointer value parameter arrives in the long class, is stored into an eight-byte local slot, and may be reassigned without changing the actual. A pointer `VAR` parameter is a reference to pointer storage and may change the actual. A pointer result uses the long return class. This follows Project Oberon and OBNC's treatment of pointer value parameters as local pointer values, while arrays and records retain Slice 12's read-only reference convention.

The module interface needs no new member kind. Exported pointer types, constants equal to `NIL`, pointer variables, and procedures with pointer parameters or results use the existing type, constant, variable, and procedure members. Cloning an interface shares pointer and record descriptors, so a client sees the same pointer identity and base-record identity. An exported pointer may expose marked fields of a private base record without exporting a name for that record, following Slice 12's rule for exported variables of private record type.

## IR and QBE lowering

The IR scalar type gains a pointer form. Its storage size and alignment are eight, its calling class is QBE `l`, and its memory operations are `loadl` and `storel`. The value form gains the zero pointer immediate used by `NIL`. QBE pointer equality and inequality use `ceql` and `cnel`; no pointer ordering instruction is reachable.

The IR gains `CheckNil`, carrying one pointer value, and `Alloc`, carrying a destination temporary, byte size, and scanned-versus-atomic choice. `CheckNil` emits `call $oberon_check_nil(l value)`. `Alloc` emits a long result from `oberon_alloc` or `oberon_alloc_atomic` with the size passed in the long class, matching C `size_t` on the supported target. Keeping allocation and the nil check explicit makes both interesting paths visible in an IR dump and prevents semantic runtime operations from being disguised as arbitrary external calls.

No new address variant is needed. After `CheckNil`, a pointer temporary can be used as `Addr::Temp` by the following field, index, load, store, whole-record copy, or structured-argument instruction. The semantic dereference helper is the only place that performs that checked value-to-address transition.

Pointer fields exercise Slice 12's layout with alignment eight. A `CHAR` followed by a pointer has seven bytes of padding, an array of pointers has stride eight, a pointer global is `align 8` with size eight, and a pointer local uses `alloc8`. Existing four-byte and one-byte layouts remain unchanged.

## Procedures and predefined operations

Pointers work in every legal role available now: module variables, locals, fields, array elements, value parameters, `VAR` parameters, and function results. The constant `NIL` works in constant declarations, assignment, value actuals, results, and equality. A record reached through a pointer works wherever that record designator already works, including whole-record assignment, structured parameter passing, field selection, array-field indexing, string assignment to a character-array field, and `LEN` in an executable expression.

`NEW` is the only new predefined procedure. Its name participates in ordinary universe-scope shadowing, so a source declaration can shadow it just as it can shadow `CHR` or `ASSERT`.

## What remains unsupported after this slice

- Record extension, inherited pointer extension beyond identical base records, extension assignment and parameter passing, `IS`, type guards, and the record and pointer forms of `CASE` remain in Slice 15.
- Open array formals and their dynamic lengths remain in Slice 14.
- Procedure types, procedure values, indirect calls, and the procedure-type uses of `NIL` remain in Slice 16.
- Heap objects carry no runtime type descriptor until Slice 15 needs dynamic type operations.
- The language has no manual deallocation, weak references, finalizers, or allocator selection. `NEW` and garbage collection are the whole storage-management interface.
- The optional `SYSTEM` module remains outside the core roadmap.

The record layout, field visibility, structured-parameter convention, executable array bounds checks, string rules, module initialization, and all existing scalar representations remain unchanged. Required-constant fixed `LEN` now accepts a dynamically selected fixed array after type-checking the selector; this is the one deliberate correction to an earlier slice.

## Changes by file

### src/ast.rs

Add a pointer type expression with its base type and position, a dereference selector with its position, and a `NIL` expression. Update type positions and designator display so diagnostics print explicit carets.

### src/parser.rs

Parse `POINTER TO type` through the full source-type production, parse `^` into a selector instead of an unsupported diagnostic, and parse `NIL` as a factor. Extend parser shape tests with named, qualified, forward, recursive-field, and inline-record pointer bases; repeated and mixed selectors; and `NIL`. Remove only the pointer and dereference assertions from the existing unsupported-construct test; record extension, procedure types, open arrays, and `IS` stay stable.

### src/sema.rs

Add the pointer descriptor and its pending, resolved, and invalid base states, the `NIL` pseudo-type and constant, pointer layout and IR mapping, shallow type display, and pointer identity. Extend type-declaration processing with a per-scope pending-base list and resolve it after the `TYPE` section against that scope alone.

Add pointer value compatibility to assignment, value arguments, results, and equality while keeping exact identity for `VAR` arguments. Mirror every rule in the constant-expression checker and evaluator, including named `NIL` constants and folded `NIL` equality.

Add explicit and implicit pointer dereference to the runtime place walk with one shared helper, one load, one nil check, base-record selection, and read-only propagation. Replace the required-constant fixed-`LEN` selector restriction with a type-only designator walk that supports direct arrays, record fields, and pointer dereference. It type-checks every index, diagnoses an out-of-range constant index, accepts a dynamic INTEGER index, and emits no runtime work.

Install `NEW` and lower its writable pointer actual through the existing variable-actual and place machinery. Compute and store each record's recursive pointer-containment property, emit the allocation kind and exact base-record size, and store the returned pointer. Ensure an invalid pending base cannot reach an internal panic during later diagnostic collection.

### src/ir.rs

Add the eight-byte pointer scalar type and zero pointer value. Add explicit nil-check and allocation instructions. Extend scalar size and storage alignment for pointer variables, fields, arrays, slots, parameters, and results.

### src/qbe.rs

Emit pointer values in the long class, pointer memory traffic with `loadl` and `storel`, pointer equality with `ceql` and `cnel`, `CheckNil` as the runtime check call, and `Alloc` as a long-class call to the selected wrapper. Existing support for `alloc8` becomes reachable for pointer-aligned slots.

### runtime/oberon.c

Add `oberon_check_nil(const void *pointer)`, which prints `nil pointer dereference` and exits for a null pointer. Change `oberon_init` to call `GC_set_all_interior_pointers(1)` before `GC_INIT`, so a selected address held across allocation keeps its containing object alive independently of the collector's build default. Keep `oberon_alloc` and `oberon_alloc_atomic` as the only allocation calls used by generated code; their signatures and behavior need no change.

### agents/architecture.md

Add pointer representation and compatibility, forward base resolution, `NIL`, explicit and implicit checked dereference, pointer parameter and result ABI, heap object layout, recursive pointer-containment classification, scanned and atomic allocation, conservative roots, and the explicit interior-pointer setting needed while a selected allocation target is live. State that no heap header or runtime type descriptor exists yet.

### Files intentionally unchanged

`src/lexer.rs` already recognizes `POINTER`, `NIL`, and `^`. `src/driver.rs` already calls the runtime initializer before module bodies and links the existing wrappers. `Cargo.toml` needs no dependency change.

## New corpus modules

`tests/corpus/` compiles each module, runs it, and compares standard output.

- `Pointers.Mod` covers a `NIL` constant, pointer module variables and locals, a procedure-local forward pointer and record pair, fields and arrays of pointers, aliases, distinct pointer constructors with the same base, assignment, both equality operators, value and `VAR` parameters, pointer results, `NEW` on a variable, a selected pointer field, and an array element whose index procedure increments a counter, explicit dereference, implicit dereference, fixed `LEN` folded through both pointer-selector forms and through a pointer selector with an earlier-scope variable index, executable `LEN` through a pointer, whole-record assignment through pointers, and local reassignment of a value parameter without changing its actual. It declares and allocates an inline record base end to end. It copies a fixed array of pointers by whole-array assignment and transports it through fixed-array value and `VAR` parameters, then reads the pointees after allocation pressure. Two simultaneously live empty-record allocations are non-`NIL` and unequal. Folded `NIL` equality is printed beside the equivalent runtime pointer comparison, and an inner pointer declaration proves that an already visible outer record name wins over a same-named record declared later in the inner scope.
- `LinkedList.Mod` declares its pointer before its node record, builds and traverses a list, inserts and removes nodes through `VAR` pointer parameters, uses `(p # NIL) & ...` to guard implicit dereference, and uses both `p^.field` and `p.field`.
- `PointerTree.Mod` uses mutually recursive pointer-bearing declarations to build a binary tree, traverses it recursively through pointer value parameters, returns a pointer from a search function, and returns `NIL` on a miss.
- `PointerLayout.Mod` places pointers after CHAR and BYTE fields, nests an array of pointers inside a record, allocates the record, writes every field before reading it, and prints values reached through offsets and array strides that would fail under a four-byte pointer layout.
- `PointerGc.Mod` allocates a pointer-free record, a record containing direct and nested pointer fields, and a record containing a zero-length array of pointers. It retains a list solely through generated pointer variables and fields, allocates hundreds of thousands of unreachable temporary objects, and traverses the retained list afterwards. It performs `NEW` on a selected field and on a heap-record array element whose index procedure records one call, so the selected address is live across a collection-capable allocation and every selector is evaluated once before that allocation. Its IL proves that the first and zero-length cases are atomic while the actual pointer-bearing layouts are scanned, and a direct run with collector statistics proves collections occurred.
- `PredefinedShadow.Mod` is extended so a source declaration can shadow `NEW` while the predefined identifier remains available in scopes where it is not shadowed.

`tests/corpus/modules/pointer-api/` supplies the cross-module coverage.

- `PointerTypes.Mod` exports a record type, a pointer type bound to it, a pointer variable initialized by its module body, a `NIL` constant, and procedures with pointer value, `VAR`, and result positions. It also exports a pointer type whose base record name is private, whose public field remains selectable, and whose private nested pointer field makes its allocation require scanning.
- `PointerApi.Mod` imports it, declares a second pointer constructor bound to the exported record, assigns and compares values across those two constructors, calls the exported procedures, reads public fields through both exported pointer types, uses the imported `NIL` constant, and calls `NEW` on the exported pointer with the private base. The last case proves the shared hidden descriptor still supplies the base size and allocation kind without exporting the record name or its private fields.

`tests/errors/` must fail with exact diagnostics.

- `PointerBad.Mod` covers a pointer bound to each non-record type available now, explicit dereference of a non-pointer, a field selected from a pointer whose base lacks it, pointer arithmetic, ordering, condition and `CASE` use, assignment and equality between pointers with different record bases, assignment of `NIL` to a non-pointer, and `NIL` in arithmetic.
- `PointerForwardBad.Mod` covers a missing forward base, a forward base that later denotes an array or pointer, a qualified missing base that must not become pending, a module pointer whose supposed base is declared only inside a procedure, and an ordinary non-pointer forward field. The existing `RecordRecursive.Mod` keeps its exact diagnostic for a record containing itself by value.
- `PointerParamBad.Mod` covers `NEW` with wrong arity, a non-variable, a non-pointer, and a read-only target; `NIL` as a `VAR` actual; different pointer constructors with the same base passed to a `VAR` formal; incompatible pointer value actuals and results; and a function returning a non-pointer where a pointer is required.
- `PointerLenBad.Mod` proves that a bad field, a non-INTEGER index, and an out-of-range constant index in fixed `LEN` are still diagnosed through both implicit and explicit pointer selectors.
- `tests/errors/modules/pointer-import-write/` proves that an imported pointer variable cannot be assigned, passed as a `VAR` pointer actual, used as the target of `NEW`, or used to mutate its referenced record, and that the same read-only propagation applies to a pointer field reached through an imported record.
- `tests/errors/modules/pointer-private/` proves that an unmarked field remains invisible through an imported pointer, including through an exported pointer whose base record type itself is private.

`tests/failures/` compiles each module, runs it, and compares standard error after a nonzero exit.

- `NilDerefExplicit.Mod` assigns `NIL` to a pointer and dereferences it with `^`.
- `NilDerefImplicit.Mod` assigns `NIL` and selects a field with the implicit dereference.
- `NilDerefChain.Mod` allocates the first node, explicitly assigns `NIL` to its next pointer, then selects a field through that second link, proving a chained implicit selector checks each pointer rather than only the original base.
- `NilDerefLen.Mod` assigns `NIL` to a pointer and evaluates executable `LEN(p.row)`, proving the runtime expression still resolves and checks the designator even though the same fixed length is legal in a constant declaration.

Every allocated record field read by a positive test is initialized first, so the test suite does not invent an initialization guarantee for either scanned or atomic allocation.

## Verification

1. Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `git diff --check`.
2. Run every new positive binary directly. Confirm a zero exit status, empty standard error, and byte-for-byte expected standard output.
3. Inspect `build/PointerLayout.ssa`. Confirm pointer globals use alignment and size eight, pointer locals use `alloc8`, pointer fields after one-byte fields have the predicted padding, pointer arrays use stride eight, and every pointer load and store uses `loadl` and `storel`.
4. Inspect `build/Pointers.ssa`. Confirm pointer value parameters and results use QBE `l`, a pointer value parameter is copied with `storel` into its own slot, a pointer `VAR` parameter receives the address of caller pointer storage, and assignment between distinct pointer constructors with one base needs no conversion.
5. Inspect the explicit, implicit, and chained dereference IL. Confirm each dereferenced pointer is loaded once, `oberon_check_nil` is textually before the first use of that value as an address, `p^.field` checks once, and `p.next.field` checks both `p` and `p.next` in order.
6. Run the four nil-failure binaries and confirm each exits nonzero with exactly `nil pointer dereference` on standard error. Confirm the guarded list traversal reaches no check on its short-circuited path. Confirm both pointer-selector forms of fixed `LEN` and the variable-index form fold in constant declarations without emitted selector code, while the executable nil case still checks and executable indices still evaluate.
7. Inspect `build/PointerGc.ssa`. Confirm `NEW` of the pointer-free record and the record whose only pointer element lies in a zero-length array call `oberon_alloc_atomic`, while every record with a pointer occupying storage at any nesting depth calls `oberon_alloc`, and each call passes the exact record payload size with no hidden header. Inspect `build/Pointers.ssa` to confirm each empty-record allocation passes size zero rather than inventing a payload byte, and run `Pointers` to prove two live empty allocations have distinct non-`NIL` identities.
8. Run `PointerGc` once normally for the corpus result and once with `GC_PRINT_STATS=1`. Confirm the statistics report at least one collection and the retained list still traverses correctly after allocation churn. Confirm selected `NEW` targets are evaluated once before their allocator calls and remain valid across them. Confirm `oberon_init` enables arbitrary interior pointers before collector initialization and no C variable or test-only runtime hook retains the Oberon heap graph.
9. Search generated IL and compiler source to confirm generated allocation names only `oberon_alloc` and `oberon_alloc_atomic`, and that BDWGC entry points remain confined to `runtime/oberon.c`.
10. Compile the cross-module gate. Confirm pointer and base-record descriptors are shared rather than rebuilt, a client pointer constructor bound to the exported record is value-compatible with the exported pointer, exact exported pointer identity is still required for a `VAR` call, and private fields stay absent. Confirm `NEW` through the exported private-base pointer uses that hidden record's exact size and scanned allocation kind.
11. Rebuild unchanged INTEGER, SET, REAL, string, array, and record programs and compare their IL with the pre-slice results. Adding the long scalar class and pointer instructions must not alter pointer-free code generation.
12. Confirm every pending pointer base is resolved or diagnosed before variable declarations and statement lowering, and compile each invalid-forward module to prove later uses produce diagnostics rather than an unsupported message or internal panic.
13. Confirm that no newly valid pointer program can reach an unsupported diagnostic, panic, QBE parse error, assembler error, or linker error, and that open arrays, record extension and dynamic type operations, and procedure types stop at their stable Slice 14, 15, and 16 diagnostics.

## Order of work

1. Parse pointer types, explicit dereference, and `NIL`, add their AST forms, and update parser shape and unsupported-boundary tests.
2. Add pointer and `NIL` semantic types, the pointer descriptor state, eight-byte layout, shallow formatting, identity, and per-scope forward-base fixups with invalid-state recovery.
3. Add the pointer IR scalar and immediate, long-class loads, stores, calls, results, and equality lowering, and exercise pointer storage in globals, slots, fields, and arrays.
4. Add `NIL` constants, pointer value compatibility, assignment, equality, value and `VAR` arguments, results, module-interface transport, and the mirrored constant-expression rules.
5. Add the shared checked-dereference helper to the runtime place walk and a type-only fixed-`LEN` designator walk, including implicit field dereference, read-only propagation, dynamic-selector constant folding, constant-index validation, and executable-`LEN` checks.
6. Compute record pointer containment, install `NEW`, add the explicit allocation instruction, select the scanned or atomic wrapper, and store allocation failure as `NIL` without a separate path.
7. Add the positive, negative, runtime-failure, cross-module, GC-pressure, and boundary gates, and add a regression module for every bug found during implementation.
8. Update the architecture document and complete the verification list.
