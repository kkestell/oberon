# Slice: Record extension and dynamic type operations

## Context

The compiler now has record identity, exact field layout, field visibility across modules, whole-record copying, pointer types, garbage-collected allocation, and structured parameters that travel by address. The semantic analyzer has also been separated into cohesive child modules without changing those rules. Record constructors still have no base type, heap records still have no dynamic type metadata, record and pointer compatibility stops at identity or an identical pointer base, and the parser still rejects record extension and `IS`.

This slice implements the roadmap's record-extension slice. It adds derived records, the inherited extension relation for pointer types, assignment from an extension to a base, polymorphic record `VAR` parameters, `IS`, designator type guards, and record and pointer `CASE`. It also introduces the smallest runtime type representation that makes those operations correct for heap records and for record variables passed through a base-type formal.

Report sections implemented: section 3 for `IS` and the type-guard selector; section 4 for visibility of imported base types and declarations used as type labels; section 6 and section 6.3 for record extension, inherited fields, and public and private field visibility; section 6.4 for the extension relation inherited by pointer types; section 7 for variables of derived types; section 8.1 for type guards; section 8.2 and section 8.2.4 for `IS`; section 9.1 for record and pointer assignment compatibility; section 9.2 and section 10.1 for derived record actuals passed to base-record value and `VAR` formals; section 9.5 for record and pointer type cases; and section 11 for extension hierarchies crossing module boundaries.

The starting point is commit `0ce12f2` (`add language intro doc`). Its compiler state comes from commit `66c1733` (`split semantic analysis into a module directory`), the structural refactor described by [the semantic-analysis module plan](2026-08-03-015-semantic-analysis-module-structure.md). The full gate passes: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test`, including 38 unit tests and the two integration tests. This slice builds on the resulting `src/sema/` module structure.

## Extension identity and compatibility

Each `RECORD` constructor still creates one type identity. A derived record descriptor additionally holds its direct base record. The extension relation is reflexive and transitive: a record extends itself and every record reached by repeatedly following its direct base. There is no depth limit.

A pointer type inherits the relation of its bound record. A pointer bound to `Derived` extends every pointer type bound to `Derived` or one of its bases, regardless of whether the pointer constructors themselves are identical. This preserves the current rule that separately constructed pointers with one identical record base are value-compatible. It widens assignment and value-parameter compatibility so a derived pointer value may be stored in a base pointer. Pointer equality and inequality accept two pointer types when either extends the other, plus the existing `NIL` cases. A pointer `VAR` formal continues to require the identical pointer type because Report 10.1's record exception does not apply to pointers.

Record assignment accepts a source whose static type extends the destination's static type. It copies exactly the destination record's size, which is the base prefix common to both types. The derived source's added fields are not copied. Assigning through a base-record `VAR` formal likewise copies that formal base prefix, even when the actual variable has a derived dynamic type; it neither changes the actual's dynamic type nor touches its derived tail.

Project Oberon implements that static-prefix rule directly. OBNC additionally traps when the source's dynamic type does not extend a polymorphic destination actual's dynamic type. That extra check is not adopted: Report 9.1 states compatibility in terms of the source and destination designators' types, and assigning the declared base prefix cannot invalidate or overwrite the actual's derived tail.

A base-record value or `VAR` formal accepts an actual record whose static type extends the formal type. This follows Report 9.2: a value actual must be assignment-compatible with its formal, so the record-extension rule from Report 9.1 applies. A structured value actual still travels as one read-only address. The callee views only the declared base prefix and cannot apply `IS` or a record guard because the formal is not a `VAR` parameter. A `VAR` actual remains writable and additionally carries its dynamic descriptor. Array parameter compatibility retains its existing exact-type and open-array rules.

Declined: structural compatibility between unrelated records, contravariant pointer assignment, and relaxed pointer `VAR` compatibility. The Report defines one nominal extension chain, not shape compatibility.

## Derived record layout and fields

A derived record begins with the complete representation of its direct base, including the base's tail padding. New fields begin at `base.size` and follow the existing alignment rule. The derived alignment is the maximum of the base alignment and every new field alignment. The final size is rounded to that alignment. This makes every base a stable byte-for-byte prefix and keeps arrays of the derived type correctly strided.

The derived descriptor stores only its directly declared fields. Field lookup searches the derived record first and then walks its bases. Each field retains the module that declared its owning record and its own export mark, so an inherited field is visible under the same rule as a direct field. A private field of a base declared in the current module participates in duplicate detection. A private field of an imported base is invisible and may therefore be redeclared by the derived record, matching the visibility provided by the reference compilers' imported interfaces.

A visible inherited field may not be redeclared. Its offset remains the offset assigned by the base. New fields cannot overlap the base prefix. Pointer containment is inherited from the base and combined with the new fields, so a derived record whose only pointer lies in a base still uses scanned allocation.

The base in `RECORD (Base)` must be a qualified identifier resolving to an already declared record type. The ordinary textual-scope rule applies. A base cannot be forward-declared, an array or pointer is not a record base, and an inline constructor cannot appear in the base position. Local record extension is accepted because the Report permits it; Project Oberon's target-specific rejection of local extension is not adopted.

The existing one-gibibyte object-size limit applies to the complete derived payload. Layout checks the inherited prefix and every newly aligned field without overflowing. An empty derived record may have the same zero size as an empty base, while still having a distinct type identity and runtime descriptor.

## Runtime type descriptors

Every successfully resolved record constructor gets one internal descriptor symbol. The symbol is based on its defining module and a module-local ordinal rather than its source name, so anonymous records, local records, aliases, and repeated local names cannot collide. An alias shares the existing record descriptor and emits no new runtime object. An imported interface shares the same descriptor symbol as the defining module.

The emitted runtime descriptor is one pointer-sized word naming the direct base descriptor, or zero for a root record. A dynamic type test starts at the actual descriptor and walks direct-base pointers until it finds the target descriptor or reaches zero. Descriptor addresses are the runtime type identities. There is no global registry, hash, numeric type identifier, fixed extension-depth table, or linker-visible source name.

The IR module owns the descriptors defined by that source module. QBE emits each as an eight-byte-aligned data object containing either `l $base_descriptor` or `l 0`. QBE accepts symbol addresses in data definitions, including a base emitted by an earlier dependency module. The driver includes descriptor words in the program-wide static-data limit.

### Heap records

An allocated record gains one hidden pointer-sized header immediately before its source payload. The header stores the descriptor of the record bound to the pointer passed to `NEW`. Generated pointers continue to point at the first source field, not at the header, so all existing field offsets, record sizes, copies, pointer comparisons, and nil checks remain source-layout operations.

The allocation wrappers take the payload size and descriptor address and reserve the header plus the payload. They first check the allocator result and return null unchanged on allocation failure. Only a nonnull result receives the descriptor header and advances to the payload address, so the existing `NEW` failure behavior still stores `NIL`. A zero-size payload reserves one additional hidden byte so the returned address lies inside the allocation, remains distinct for simultaneous empty objects, and is recognized as an interior pointer by BDWGC. That byte is not part of the Oberon record: layout size and copies remain zero. The pointer-sized header preserves every alignment the current type system can request.

`oberon_alloc` and `oberon_alloc_atomic` remain the only generated allocation entry points. The scanned-versus-atomic choice still comes from the complete source payload. A descriptor pointer in an atomic object's header does not point into the GC heap and does not require scanning. Arbitrary interior-pointer recognition remains enabled before collector initialization because every returned payload pointer is now an interior pointer, in addition to the selected addresses that already required the setting.

### Polymorphic record parameters

A `VAR` record formal expands to two machine arguments: its existing long address followed immediately by one long descriptor address. A value record formal remains one address even when a derived actual is accepted. The callee can access only the formal's declared base prefix, and the language does not permit a value record formal as the subject of `IS`, a record guard, or a record type case, so its dynamic type is unobservable. Source arity and interface signatures remain expressed in source parameters; only typed IR expands the machine convention.

An ordinary global, local, array element, or record field contributes its statically known descriptor when passed to a `VAR` record formal. A polymorphic record formal forwarded to another call contributes its incoming descriptor. A record reached through `p^` contributes the descriptor stored in that heap object's header. Selecting a field or array element from a polymorphic record returns to the selected component's static type, because extension changes the outer record and does not change the declared type of one of its components.

The resolved-place model therefore gains optional dynamic record metadata. It distinguishes a static descriptor, an incoming descriptor argument, and a heap descriptor recoverable from an already checked pointer. The metadata survives a record guard and a type-case narrowing of the same variable. It is replaced with the selected component's static descriptor after a field or index selector. Materializing a heap descriptor is an explicit IR operation that loads the hidden header after the existing nil check.

## `IS`

The parser represents `v IS T` as a type-test expression whose right side is a qualified type name, not as an ordinary binary expression. Semantic analysis resolves the left operand exactly once and accepts these forms:

- An arbitrary pointer-valued expression tested against a pointer type that extends its static pointer type.
- A record `VAR` formal tested against a record type that extends its declared record type.
- `NIL`, including a constant whose value is `NIL`, tested against a pointer type.

An ordinary record variable, structured value parameter, array element of record type, nonpointer scalar expression, procedure, or unrelated target type is rejected. A type name must match the subject's record-versus-pointer form. Testing against the declared type itself is valid because the extension relation is reflexive.

The Report leaves `NIL IS T` undefined. Project Oberon treats `NIL` as belonging to every pointer type, while OBNC and oberonc make the test false. This compiler defines it as `FALSE`: `NIL` denotes no allocated object and therefore has no dynamic record type. The same rule applies to a named `NIL` constant. It folds in a constant expression without allocating or trapping.

A non-`NIL` type test is a pure Boolean operation. It never traps merely because the dynamic type does not match. The pointer expression is evaluated exactly once, including a procedure call that returns a pointer. Ordinary selectors needed to obtain a tested pointer still keep their existing effects and checks; for example, a nil dereference or an out-of-range index before `IS` fails in the ordinary way. A record formal's descriptor is likewise captured once.

## Designator type guards

A guard `v(T)` is accepted under the same subject and extension rules as `v IS T`, except that `NIL` is not a designator and cannot be guarded. A successful guard keeps the same storage address and read-only state while narrowing the designator's static type to `T` for later selectors and for the context consuming the complete guarded designator. A record guard preserves the incoming dynamic descriptor. A pointer guard preserves the pointer storage and heap metadata; a later implicit or explicit dereference still performs the ordinary nil check.

A false guard calls a new runtime trap that prints `type guard failed` and exits with nonzero status. Guarding a nil pointer therefore fails with this guard message rather than the nil-dereference message. The type check occurs before any selector following the guard. The base designator and all selectors before the guard are evaluated once.

Parentheses are syntactically ambiguous because `p(T)` can be either a procedure call or a terminal type guard until names have types. The parser must preserve that ambiguity instead of guessing from spelling. A parenthesized postfix followed by another selector is necessarily a type guard and becomes a guard selector after its contents are checked to be one qualified identifier. A terminal parenthesized postfix stays a neutral application node. Semantic analysis interprets it as a call when the prefix denotes a procedure or predefined operation, and as a guard when the prefix denotes an eligible record or pointer designator and the parentheses contain exactly one qualified type name. Statement parsing likewise keeps enough postfix structure to recognize `p(T).field := value` and a guarded designator used as an actual parameter without misclassifying existing calls such as `F(x)`.

Constant-expression and required-constant `LEN` walks recognize the new syntax. A dynamic type guard is not a constant operation and cannot be skipped merely because a selected fixed-array length is known. A constant expression containing one receives the ordinary nonconstant diagnostic. The existing type-only `LEN` path remains unchanged for pointer dereference, field, and index selectors that do not contain a guard.

## Record and pointer `CASE`

The scalar `CASE` form continues to accept an arbitrary `INTEGER` or `CHAR` expression, range labels, multiple labels per arm, overlap checking, and the existing no-match trap.

The type form accepts a qualified variable with no value selectors. It must denote a pointer variable or a record `VAR` formal. This follows the Report's description of a "case variable" and Project Oberon's restriction to a qualified identifier, and it gives the narrowing a single stable symbol to affect. A pointer case captures the pointer value once. A record case captures the formal's incoming descriptor once.

Each type arm has exactly one qualified type label and no range. The label must have the same record-versus-pointer form as the case variable and must extend its declared type. Duplicate labels are rejected. Labels that are related but not identical are allowed, and arms are tested in source order; a base label placed first may therefore subsume a later derived label.

Within one matching arm, references to the case variable use the label type. Its address, incoming dynamic descriptor, read-only state, and declared procedure signatures do not change. The analyzer restores the original type before checking the next arm and after the statement, including for nested type cases. The narrowing applies only to uses of that variable in the arm; it does not mutate an exported interface or the formal type recorded in any procedure signature.

If no type arm matches, the statement does nothing. Project Oberon, OBNC, and oberonc all lower type cases as conditional chains with a fallthrough, unlike their scalar case paths. This slice follows that shared behavior and does not call `oberon_case_no_match` for a type case. A nil pointer therefore matches no arm and falls through.

The grammar permits an empty case alternative. The current parser omits empty alternatives from the AST, and the type-case path keeps that representation. `CASE p OF | P1: statements END` behaves as a type case with only the `P1` arm, while `CASE p OF END` is a no-op when `p` selects the type-case path. Empty alternatives do not suppress validation of any nonempty arm. Scalar `CASE` retains its existing empty-alternative and no-match behavior.

## AST and parser

`ast::TypeExpr::Record` gains an optional base qualified identifier. `ast::Selector` gains a type-guard form. The expression AST gains explicit type-test and neutral terminal-application forms sufficient to defer the call-versus-guard decision to semantic analysis. `BinOp` does not gain `IS`, because the right operand is a type name and the operation has different applicability rules from value relations.

The parser accepts `RECORD (Base) ... END`, `IS` followed by a qualified identifier, guards at any selector position, and the existing procedure-call forms. Parser tests pin the shapes of a root and derived record, a qualified base, repeated guards, guards followed by fields, indices, and dereference, terminal call-or-guard applications, and `IS`. Malformed guards cover an empty list, several arguments, an expression instead of a qualified identifier where a following selector makes the guard syntactically certain, and a missing right parenthesis.

The parser continues to parse the general `CASE expression` production. Semantic analysis selects scalar or type behavior after it knows the case expression's type and diagnoses a type case that is not a qualified variable.

## Semantic types and lowering

`src/sema/types.rs` owns the direct-base link, extension queries for records and pointers, inherited field lookup, compatible pointer equality, and the widened assignment outcomes. It gives record prefix copying a distinct assignment outcome so lowering uses the destination size rather than accidentally copying the larger source. It also owns the runtime descriptor symbol stored in each record descriptor.

`src/sema/symbols.rs` extends variable symbols with the optional incoming descriptor carried only by a polymorphic record formal. Exported interfaces need no new member kind: their existing shared `Type` values retain base links and descriptor symbols.

`src/sema/constant.rs` type-checks the new AST forms without emitting code. It folds `NIL IS PointerType` to `FALSE`, rejects all other dynamic tests as nonconstant when a constant is required, and rejects guards in required-constant selector walks.

`src/sema/mod.rs` resolves derived layouts, assigns module-local descriptor ordinals, expands `VAR` record parameters, carries dynamic metadata in places, widens record and pointer compatibility, lowers tests and guards, and adds the type-case path beside the scalar path. The call builder passes one descriptor after each `VAR` record address, independently for every formal, just as each open array already expands independently into its lengths.

Evaluation order remains explicit. Assignment resolves the destination before the source. Calls resolve actuals from left to right. A pointer or descriptor used for `IS`, a guard, or a type case is obtained once. A failed guard occurs before any following selector or arm body. No descriptor operation reloads or reevaluates an Oberon designator behind semantic analysis's back.

## Typed IR, QBE, and runtime

`src/ir.rs` adds module-owned runtime descriptor data, an allocation descriptor operand, a heap-descriptor load, and explicit dynamic type-test operations for a pointer value and for a descriptor value. The IR continues to distinguish source record storage from runtime metadata: `Storage::Record` remains payload size and alignment only. Reference parameters and arguments already use the long class, so the hidden descriptor argument needs no new machine type.

`src/qbe.rs` emits descriptor data, passes descriptor addresses in the long class, changes allocation calls to include the static record descriptor, loads a heap descriptor from the hidden header when requested, and lowers the two type-test forms to the runtime helpers. Existing field offsets and `CopyBytes` counts do not include the header. Programs that declare no record type and allocate no record should retain byte-identical QBE IL.

`runtime/oberon.c` defines the one-word descriptor shape and the base-chain test. It changes both allocation wrappers to install the hidden header and return the payload. It adds helpers for testing an actual descriptor and an allocated pointer, and adds the `type guard failed` trap. The helpers treat a null pointer as a false test and never dereference it. BDWGC remains confined to this file.

`src/driver.rs` counts emitted descriptor words along with globals and string literals when enforcing the program-wide static-data limit. No source lookup, module ordering, command invocation, or interface cache behavior changes.

## Architecture documentation

Update [architecture.md](../architecture.md) after implementation. Replace the statement that record extension is absent with the nominal extension and inherited-field rules. Document base-prefix layout, record and pointer compatibility, the address-plus-descriptor convention for record `VAR` parameters, `NIL IS T = FALSE`, type guards, type-case narrowing and fallthrough, static descriptor data, and the heap header. Update the heap section so it no longer claims that `NEW` has no header or runtime descriptor, while preserving the source-payload sizes and scanned-versus-atomic rule.

## Corpus gate

Positive programs compile, link, exit successfully, write no standard error, and compare standard output byte for byte.

- `RecordExtensions.Mod` defines a root, child, grandchild, and sibling. It exercises inherited field offsets, inherited pointer containment, assignment from child and grandchild to each base prefix, a derived value actual passed to a base-record value formal, pointer assignment and equality across the hierarchy, guards at terminal and intermediate selector positions, local extension, an empty-record hierarchy, and `NIL IS P = FALSE`.
- `DynamicTypes.Mod` passes ordinary records, forwarded record `VAR` formals, and records reached through `p^` to a base-record `VAR` formal. It tests true and false `IS`, a side-effecting pointer-result call used as an `IS` subject, successful record and pointer guards, source-ordered pointer and record cases, empty type-case alternatives, a type case with no arms, nested cases, restoration after each arm, and the no-match behavior for a base record and `NIL` pointer. It applies `NEW` through a pointer narrowed by a guard and through one narrowed by a type-case arm, then proves that allocation uses the narrowed record's payload and descriptor.
- `RecordExtensionLayout.Mod` mixes one-byte, four-byte, and pointer fields across three levels. It proves the complete base size is a stable prefix, inherited fields keep their offsets, added fields receive correct alignment, and array stride uses the final derived size.
- `PointerExtensionGc.Mod` allocates derived objects through derived pointers, stores them in base pointers, and retains them only through generated-code roots while allocation churn forces collections. It tests dynamic descriptors after collection, inherited scanned allocation, descriptor recovery from `p^`, and distinct non-`NIL` allocations for empty derived records.
- `tests/corpus/modules/extension-api/` defines an exported hierarchy and exported procedures in one module and consumes them in another. It extends an imported base, selects inherited public fields, keeps inherited private fields hidden, passes derived records through imported base-record `VAR` procedures, performs tests and cases against imported types, and proves that descriptor identity and base links survive interfaces without duplication.

Negative modules fail compilation with exact diagnostics.

- `RecordExtensionBad.Mod` covers a missing base, a non-record base, a forward base, redeclaration of a visible inherited field, incompatible sibling and base-to-derived assignment, a base record passed to a derived structured value formal, unrelated pointer assignment and equality, and a nonidentical pointer passed to a pointer `VAR` formal.
- `TypeTestBad.Mod` covers an ordinary record variable, a structured value parameter, a scalar or procedure subject, a non-type target, a record target for a pointer and a pointer target for a record, an unrelated target, and a base target that does not extend the declared subject type.
- `TypeGuardBad.Mod` covers the same subject and target errors in selector form, several or non-name guard arguments, a guard used in a required constant `LEN`, and attempts to use a failed partial parse as a procedure call.
- `TypeCaseBad.Mod` covers a record subject that is not a `VAR` formal, a selected rather than qualified case variable, a non-record and non-pointer subject, a non-type or unrelated label, a record-versus-pointer label mismatch, a label list, a label range, and a duplicate type label.
- Cross-module errors cover selection of an inherited private field and extension from an unexported base type without leaking the private declaration's name.

Runtime-failure modules compile successfully and compare stable standard error.

- `RecordTypeGuardFail.Mod` passes a root record to a base-record `VAR` formal and applies a derived guard. It prints `type guard failed` and exits before any following field access.
- `PointerTypeGuardFail.Mod` guards a nonnil sibling object as the wrong derived pointer type.
- `NilTypeGuardFail.Mod` guards `NIL` and proves the failure is the type-guard trap rather than a nil dereference.

Existing record, pointer, open-array, module, and GC programs remain in the gate. `Pointers.Mod` continues to prove that two live empty allocations are distinct. `PointerGc.Mod` continues to prove that source payload scanning and arbitrary interior roots survive collection. The new header must not change any source-visible record size, field offset, copy count, array stride, or output.

## Verification

1. Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `git diff --check`.
2. Run every new positive and runtime-failure binary directly. Confirm the expected status, byte-for-byte output, and empty standard error for successful programs.
3. Inspect the parser AST tests and diagnostics for the ambiguous `p(T)` form. Confirm existing one-argument procedure calls still lower as calls, terminal guards lower as guards after name resolution, and a guard followed by another selector never becomes a call.
4. Inspect derived record descriptors in semantic traces. Confirm one direct-base link per constructor, shared identity through aliases and imports, no duplicate descriptor for an alias, and one internal symbol for every anonymous or local constructor that needs one.
5. Inspect `build/RecordExtensionLayout.ssa`. Confirm inherited field offsets are unchanged, new fields begin no earlier than the base size, globals and slots reserve only payload bytes, arrays stride by the derived payload size, and a derived-to-base assignment passes the base destination size to `oberon_copy`.
6. Inspect procedure signatures and calls in `build/DynamicTypes.ssa`. Confirm each `VAR` record formal receives address then descriptor, each structured value formal still receives only an address when its actual is the identical or a derived record type, ordinary actuals pass static descriptors, forwarded formals pass incoming descriptors, and `p^` loads its heap descriptor only after the pointer's nil check.
7. Inspect descriptor data in the cross-module IL. Confirm each root contains zero, each derived descriptor points at its direct base symbol, imported and aliased types reuse the defining symbol, and descriptor bytes are included in the driver's static-data total.
8. Inspect allocation IL and both runtime allocation wrappers. Confirm `NEW` passes payload size and the currently narrowed bound record descriptor to `oberon_alloc` or `oberon_alloc_atomic`; field offsets, copy sizes, and pointer values remain relative to the returned payload. Confirm each wrapper returns a null allocator result before writing the header or applying payload pointer arithmetic. Confirm pointer-free records, including a derived record with no inherited pointer, still use the atomic wrapper, while a pointer inherited from any base selects the scanned wrapper.
9. Run `PointerExtensionGc` normally and with `GC_PRINT_STATS=1`. Confirm at least one collection occurs, objects retained only through base pointers survive, their dynamic tests still succeed, and pointer-free payloads continue to use the atomic allocation wrapper despite their metadata header.
10. Exercise zero-size root and derived allocations. Confirm every result is non-`NIL`, simultaneous objects compare unequal, their `IS` results use the correct descriptor, and record copies still move zero bytes.
11. Inspect every `IS`, guard, and type-case path. Confirm the subject is evaluated once, including a pointer-returning call with a visible side effect; false tests return `FALSE`; `NIL IS P` is `FALSE`; failed guards trap before later selectors; type-case arms test in source order; empty alternatives contribute no arm; and an unmatched or armless type case emits no scalar-case trap.
12. Rebuild representative pre-slice modules and compare their IL. Programs without record declarations or allocation should remain byte-identical. Programs with records should differ only by descriptor data, hidden `VAR` record descriptor arguments where applicable, and descriptor-aware allocation where `NEW` occurs.
13. Search for the old unsupported diagnostics and stale architecture claims. Correct source using record extension, `IS`, type guards, and type cases must not reach an unsupported diagnostic, panic, QBE error, assembler error, or linker error. Procedure types remain the only unsupported core grammar production after this slice.

## What remains unsupported

Procedure types, procedure variables, procedure parameters, `NIL` procedure values, and indirect calls remain for the next language slice. Portable standard modules and the final conformance audit remain later roadmap work. Record equality remains absent because the Report does not define it. Array covariance, dynamic array element types, object finalizers, reflection, public runtime type names, and user-visible descriptor access are not Oberon-07 features and are not introduced.

## Order of work

1. Extend the AST and parser for record bases, `IS`, guard selectors, and the call-versus-guard ambiguity, with parser-shape and malformed-syntax tests.
2. Add record base links, extension queries, inherited field lookup, base-prefix layout, widened assignment compatibility, and unique descriptor symbols in `src/sema/types.rs` and type construction.
3. Add descriptor data and heap-descriptor operations to typed IR and QBE emission, then update the runtime allocation wrappers and type-test helpers. Verify root and derived descriptors and zero-size allocation before using them in semantic lowering.
4. Expand the record `VAR` calling convention and resolved-place metadata. Verify static actuals, forwarded formals, and records reached through `p^` independently.
5. Implement derived-to-base record copying, derived pointer assignment and equality, and extension-compatible record value and `VAR` actual checking.
6. Implement `IS` with the explicit false-on-`NIL` rule, then implement guards using the same applicability and runtime test path plus the guard trap.
7. Add the type-case lowering and scoped symbol narrowing beside the unchanged scalar case path.
8. Add the positive, negative, runtime-failure, cross-module, layout, ABI, and GC gates. Add a regression module for every bug found during implementation.
9. Update architecture documentation, run the full verification list, inspect the complete diff, and report the slice ready for independent review.
