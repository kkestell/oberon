# Slice: Procedure types and indirect calls

## Context

The starting point is commit `cbe0276` (`Add record extension and dynamic type operations`), the completed roadmap Slice 15 described by [the record-extension plan](2026-08-03-016-record-extension-and-dynamic-type-operations.md). It provides record extension, dynamic type descriptors, and the address-plus-descriptor convention for record `VAR` parameters. The full gate passes: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test`, including 41 unit tests and the two integration tests.

The compiler already represents every declared procedure with a source signature and lowers direct calls through one argument builder. Scalar value parameters travel in registers, structured parameters travel by address, open arrays add one length per open dimension, and record `VAR` parameters add a dynamic descriptor after their address. Procedure declarations themselves still cannot be used as expression values. The parser rejects procedure types, semantic types have no callable value, calls name a direct symbol in typed IR, and the backend cannot call through a computed address.

This slice implements roadmap Slice 16. It adds procedure types, procedure values, variables and parameters of procedure type, `NIL`, equality and inequality, function results of procedure type, and indirect calls. It preserves the existing source calling convention by making direct and indirect calls share the same argument construction and machine signature.

Report sections implemented: section 4 for visibility and qualified access to procedure values; section 6 and section 6.5 for procedure types and eligible procedure values; section 7 for variables of procedure type; section 8.1 for procedure designators and activation; section 8.2.4 for procedure equality and inequality; section 9.1 for assignment and `NIL`; section 9.2 for direct and indirect procedure calls; section 10 and section 10.1 for proper and function signatures, procedure parameters, recursion, globally declared actual procedures, and result restrictions; and section 11 for procedure types and values crossing module boundaries.

After this slice, every core grammar production in the May 2016 Report has an implementation. Portable standard modules and the final conformance audit remain later roadmap work.

## Procedure signatures

A procedure type records an ordered source signature. Each formal records its value or `VAR` mode and its resolved source type. The signature also records either no result for a proper procedure or one result type for a function procedure. Formal names do not participate in type equality and are not visible outside the type expression, but duplicate names within one formal-parameter list remain declaration errors.

Two procedure signatures match when they describe the same callable shape. They must both be proper procedures or both be function procedures. Function result types must be the same constructor or an alias of it. Formal counts, order, modes, and types must match. Open-array rank and element types must match. A procedure-typed formal is compared recursively by signature. Fixed arrays, records, pointers, and named scalar types retain the compiler's existing identity rules; signature matching does not introduce covariance, contravariance, record extension, pointer extension, or INTEGER and BYTE compatibility.

This structural rule follows Project Oberon for assignment, value substitution, and relations while retaining this compiler's nominal rules for the component types inside the signature. Two separately written procedure type constructors with matching signatures are therefore interchangeable in value contexts. A `VAR` formal follows Report 10.1's stricter identical-type rule: its actual must be a writable variable of the same procedure type constructor or an alias of it. Separately written matching constructors are rejected as `VAR` actuals. Signature matching compares function result types by constructor identity, following Project Oberon rather than oberonc's recursive extension of signature equality.

A named procedure declaration uses the same semantic procedure-type representation as a procedure variable. Symbols and exported interfaces pair that type with the declaration's direct code symbol instead of carrying a second, potentially divergent parameter and result model. The semantic model distinguishes constructor identity from recursive signature matching so each context can apply the Report's required rule without duplicating signature fields.

The result type of any procedure signature can be a basic, pointer, or procedure type. It cannot be an array, open array, or record. This restriction is checked while the signature is built, so an illegal result is rejected equally in a procedure declaration and in a procedure type expression.

Declined: parameter variance, result covariance, implicit INTEGER and BYTE matching inside signatures, and compatibility based only on the expanded machine ABI. Procedure compatibility is a source-language rule and remains exact even when two different source types happen to use the same QBE class.

## Procedure values and eligibility

A procedure value is one pointer-sized code address or zero for `NIL`. Procedure types occupy eight-byte storage and use the QBE long class in values, parameters, results, globals, locals, arrays, and record fields. A procedure value is not a heap pointer. It does not make a record require scanned allocation, and it adds no runtime descriptor or closure object.

Only a procedure declared in a module scope may become a procedure value. The rule includes an exported procedure reached through an imported module and a module-level procedure used recursively within its own body. A nested procedure and every predefined procedure remain callable directly but are rejected whenever a value is required. The temporary runtime-backed `Out` members are ordinary imported module procedures rather than predefined identifiers, so they follow the same eligibility rule as other imported procedures.

A direct eligible procedure designator lowers to its code symbol address when used in assignment, a relation, an argument, or a return. A procedure variable, parameter, array element, or record field lowers through its ordinary storage place. A procedure-valued function call produces the same pointer-sized value. No closure or static link is needed because nested procedures cannot become values.

`NIL` is assignment-compatible with every procedure type and compares equal to a zero procedure value. Two procedure values may be compared with `=` or `#` when their signatures match. Ordering relations remain illegal. An eligible direct procedure may be compared or passed without first storing it in a variable.

Module and heap storage keep the initialization behavior already provided by their storage class. Module data starts zeroed. Procedure locals remain undefined until assigned. Tests do not rely on the initial value of a procedure local or a newly allocated record field.

## Parameters and results

A value formal of procedure type receives one code address in the long class and behaves like the existing scalar value formals: it is copied into a local slot and may be reassigned without changing the caller's value. A `VAR` formal receives the address of a writable procedure variable. An imported procedure variable is readable and callable but cannot be assigned or passed to a `VAR` formal.

A value procedure formal accepts an eligible global procedure, a procedure variable, a procedure value parameter, a procedure-valued expression with a matching signature, or `NIL`. It rejects nested and predefined procedures even when their visible call signature would otherwise match. A `VAR` procedure formal accepts only a writable designator of the identical procedure type constructor or an alias; a direct procedure declaration, `NIL`, and a variable of a separately constructed matching type are not valid `VAR` actuals.

A function may return a procedure value. Its `RETURN` expression follows the same compatibility and eligibility rules as assignment, including `NIL`. Because the grammar permits selectors and calls only on a designator before its optional actual-parameter list, a returned procedure value must be stored in a variable before it can be activated; this slice does not add call chaining syntax that Oberon-07 lacks.

Procedure types can appear as array elements and record fields. Existing whole-array and whole-record copying copies their pointer-sized values. Existing field export and imported-variable read-only rules continue through procedure-valued selectors.

## Direct and indirect calls

Typed IR distinguishes a direct call target from an indirect value target. Both targets use the same argument list and optional result representation. Semantic lowering resolves the callable first, obtains its source signature, and then invokes the existing argument builder. Every ABI expansion remains exactly where it is today: structured values pass by address, open arrays add their lengths, and record `VAR` arguments add their dynamic descriptors.

The Report requires actual expressions to be evaluated before procedure activation but does not specify their order relative to reading an indirect call target. This compiler resolves that gap uniformly: an indirect call loads its procedure value exactly once before evaluating any actual parameter, then evaluates actuals from left to right through the ordinary argument builder. The call uses the captured value even if an actual parameter changes the procedure variable from which it came. This extends the source-order rule already used for selectors and arguments and avoids Project Oberon's representation-dependent difference between simple and computed call targets.

After all actual parameters have been evaluated, calling a captured zero procedure value invokes a dedicated runtime failure immediately before activation. The failure prints `nil procedure call` to standard error and exits with nonzero status. It does not reuse the `nil pointer dereference` trap because a procedure activation is not a data dereference. Direct calls need no dynamic check.

Proper procedure variables can be activated as statements with or without an explicit empty actual-parameter list when their signature has no parameters. Function procedure variables require the actual-parameter list in expression position, including `()` for a parameterless function, just like directly declared functions. A function-procedure designator without that list denotes the procedure value and remains valid in assignment, passing, return, and comparison contexts. A proper procedure used as a value-producing call and a function call used as a statement retain the existing diagnostics.

The call-versus-guard decision introduced by Slice 15 becomes type-directed for procedure variables as well as direct procedures. A terminal `p(T)` application is an indirect call when `p` resolves to a procedure-valued place and is a type guard when `p` resolves to an eligible record or pointer place. A call target may itself contain field, index, dereference, or guard selectors before the terminal actual-parameter list. A selector after an actual-parameter list remains invalid under the Report's grammar.

## AST and parser

`ast::TypeExpr` gains a procedure form containing formal sections, an optional result type, and the source position. The existing formal-section representation is reused so multidimensional open arrays and `VAR` modes have one syntax model. The parser accepts `PROCEDURE` with no formal-parameter list, `PROCEDURE()`, proper signatures with parameters, and function signatures whose result follows the closing parenthesis.

Procedure declarations and procedure type expressions share a small parser helper that returns the formal sections and optional result of the complete `FormalParameters` production together. Ordinary procedure headings keep their declaration name and body separately. The result remains a qualified type name because that is what the grammar permits. A result colon is accepted only after a parenthesized formal-parameter list, so a parameterless function must use `()` and both `PROCEDURE F: INTEGER` and `PROCEDURE: INTEGER` are rejected.

Parser tests pin proper and function procedure types, absent and empty parameter lists, mixed value and `VAR` sections, multidimensional open arrays, procedure types nested in arrays and records, and the distinction between a terminal application and a type guard. Malformed cases cover a missing formal type, a missing closing parenthesis, a result colon without a type, a function result without the required parameter list, and inline structure where the result grammar requires a qualified name. Semantic diagnostics cover duplicate formal names across sections of one procedure type.

## Semantic types and symbols

`src/sema/types.rs` owns the semantic procedure type, constructor identity, recursive signature matching, storage size and alignment, display, assignment compatibility, relation compatibility, and the rule that a procedure value is not a GC pointer. The representation uses shared owned types like the existing arrays, records, and pointers. Assignment, value substitution, and relations ask for matching signatures, while `VAR` substitution and a signature's function result ask for identical type constructors or aliases.

`src/sema/symbols.rs` makes a declared procedure carry its semantic procedure type, direct symbol, and value-eligibility flag. Exported procedure members carry the same shared type and symbol. Variables need no new symbol kind because their existing `Type`, address, shape, read-only state, and optional record metadata already describe procedure-valued storage.

`src/sema/mod.rs` builds procedure type expressions, builds declarations through the same signature constructor, lowers eligible direct procedures as values, and resolves direct or indirect call targets before passing their signature to one argument builder. Assignment, value and `VAR` parameters, returns, and relations use the centralized compatibility rule. Procedure-valued scalar storage follows the existing global, slot, field, and index paths.

`src/sema/constant.rs` recognizes procedure types and indirect calls during the non-emitting type walk. A procedure designator, procedure relation, or procedure call is not a constant expression. It still receives its ordinary type or call diagnostic before the required-constant diagnostic, and no constant-only designator walk executes a call or treats a code address as a constant.

## Typed IR, QBE, and runtime

`src/ir.rs` represents a callable as either a direct symbol or an indirect value. `Inst::Call` retains one optional result and one argument vector for both. A dedicated procedure-value check makes the dynamic failure explicit instead of letting a zero address reach QBE as an unchecked indirect call. Procedure values use a pointer-sized machine value without being classified as source pointers.

`src/qbe.rs` emits direct targets as `call $symbol(...)` and indirect targets as `call %temporary(...)`. The installed QBE accepts a long symbol address copied into a temporary and called through that temporary. Arguments and results retain their existing QBE classes, so an indirect call to a signature containing open arrays or polymorphic record `VAR` parameters is byte-for-byte ABI-compatible with a direct call to the same procedure.

`runtime/oberon.c` adds only the dedicated zero-procedure check and trap message. Procedure storage, equality, and calls need no registry, descriptor, trampoline, allocation, or C wrapper. BDWGC remains unaware of code addresses.

The driver, module ordering, linker invocation, source lookup, and program-wide static-data accounting need no special path. Procedure globals and aggregate fields contribute through their ordinary eight-byte storage. Imported direct procedure symbols and procedure-valued data use the existing module graph and interface identities.

## Architecture documentation

Update [architecture.md](../architecture.md) after implementation. Document recursive procedure-signature matching, the stricter constructor-identity contexts, global-procedure eligibility, pointer-sized code values, the absence of closures, procedure value parameters and results, shared direct and indirect call argument expansion, callee-before-actual evaluation, and the dedicated zero-procedure call failure. Remove procedure types from the list of unsupported grammar.

## Corpus gate

Positive programs compile, link, exit successfully, write no standard error, and compare standard output byte for byte.

- `ProcedureValues.Mod` defines proper and function procedure types. It exercises module and local variables, `NIL`, `NIL` passed to a value procedure formal, assignment from eligible procedures, assignment between separately declared matching procedure types, equality and inequality, reassignment of a value procedure parameter, a `VAR` procedure parameter used through an alias of the identical type, proper and function indirect calls, a parameterless function designator used as a value without activation, a procedure-valued function result, and direct procedure values used as arguments and relations.
- `ProcedureAggregates.Mod` stores procedures in fixed arrays and records, copies both aggregates, calls selected elements and fields, and assigns through a field reached after ordinary record and pointer selectors. It proves procedure fields do not make an otherwise pointer-free heap record select scanned allocation.
- `ProcedureCallOrder.Mod` obtains an indirect target through selectors with visible index side effects. An actual parameter then replaces the original procedure variable. The output proves that the target and its selectors are evaluated once, before actual parameters, and that the captured target is the one invoked.
- `ProcedureOpenArrays.Mod` passes a procedure whose signature contains multidimensional open arrays. It calls the same target directly and indirectly with several fixed shapes and observes every length. It also passes a callback with a base-record `VAR` formal and proves that indirect calls carry the Slice 15 dynamic descriptor after the record address.
- `tests/corpus/modules/procedure-api/` exports a procedure type, an eligible procedure, a procedure-valued variable initialized by the dependency module, a callback-taking procedure, and a procedure-valued function. A client assigns and invokes the imported procedure, reads and calls the imported variable without mutating it, and proves that matching signature types and hidden ABI arguments survive the interface. The client also assigns and invokes `Out.Char` through a procedure variable, proving that an ordinary runtime-backed module procedure is eligible while a predefined identifier is not.

Negative modules fail compilation with exact diagnostics.

- `ProcedureTypeBad.Mod` covers array and record result types, incompatible proper and function signatures, mismatched result types, parameter count, order, mode, open-array rank, named component identity, and INTEGER versus BYTE. It distinguishes separately constructed matching procedure types used as formal types, where recursive signature matching applies, from the same types used as function results, where constructor identity is required. It also covers duplicate formal names within and across sections, an ordinary nonprocedure value used where a procedure value is required, and an ordering relation on procedures.
- `ProcedureValueBad.Mod` rejects nested and predefined procedures in assignment, comparison, value-parameter, and return contexts. It rejects a direct procedure, `NIL`, and a variable of a separately constructed matching procedure type as `VAR` actuals. It also rejects assignment to a read-only imported procedure variable and use of an incompatible procedure variable as an argument or return value.
- `IndirectCallBad.Mod` covers wrong arity and argument types through a procedure variable, a proper procedure called as a function, a function called as a statement, a missing parameter list on a function designator, and a selected nonprocedure used as a call target.
- Cross-module errors cover access to an unexported procedure or procedure type and mutation of an exported procedure-valued variable, including through a selected aggregate.

Runtime-failure modules compile successfully and compare stable standard error.

- `NilProcedureCall.Mod` calls a `NIL` proper procedure variable and prints `nil procedure call`.
- `NilFunctionCall.Mod` calls a `NIL` function procedure variable in an expression and prints the same failure.
- `NilProcedureCallOrder.Mod` gives a `NIL` target an actual expression with observable output. Direct execution confirms that the actual runs before the activation fails.

Existing direct-call, nested-procedure, open-array, record-extension, dynamic-type, module, aggregate-copy, and GC programs remain in the gate. Programs that neither declare a procedure type nor use a procedure as a value retain byte-identical QBE IL.

## Verification

1. Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `git diff --check`.
2. Run every new positive and runtime-failure binary directly. Confirm successful status and byte-for-byte output for positive programs. Confirm nonzero status, exact standard error, and the specified standard output behavior for failures.
3. Inspect parser AST tests for every procedure-type spelling and the terminal call-versus-guard ambiguity. Confirm the parser preserves source modes, open ranks, result names, and application structure without resolving types.
4. Inspect semantic traces for separately declared matching procedure types, aliases, imported types, and deliberately mismatched signatures. Confirm value compatibility and relations use recursive signature matching, `VAR` actuals and function result types use constructor identity, formal names are ignored after duplicate-name checking, and no existing record, pointer, or fixed-array identity rule is weakened.
5. Inspect global and local storage for procedure variables, arrays, and record fields. Confirm each procedure value occupies eight bytes, aggregate layout and copy counts include those bytes, and procedure-only heap records still use `oberon_alloc_atomic`.
6. Inspect direct procedure values in emitted IL. Confirm eligible local-module and imported declarations become `$symbol` addresses, aliases do not create wrappers, and nested or predefined procedures never reach a value instruction.
7. Inspect direct and indirect calls with scalar, structured value, open-array, record `VAR`, and procedure parameters. Confirm both target forms use one argument order and one set of hidden expansions. In particular, every record `VAR` address is followed by its descriptor and every open dimension contributes its length in the same position for direct and indirect calls.
8. Inspect `ProcedureCallOrder.ssa`. Confirm the indirect target is loaded into one temporary before any actual is lowered, selectors occur once, actuals remain left to right, the zero check follows argument evaluation, and the final QBE call uses the captured temporary.
9. Inspect proper and function procedure results. Confirm a procedure-valued return uses the long class, a caller stores or forwards it without conversion, and an indirect function call declares the same result class as its direct target.
10. Inspect equality and inequality. Confirm direct symbols, loaded procedure values, and `NIL` compare as pointer-sized values only after semantic signature checking, and no ordering operation reaches IR.
11. Inspect both zero-call failure paths. Confirm actuals are evaluated before the dedicated check, the check immediately precedes indirect activation, uses the procedure-specific message, and cannot be confused with a pointer dereference or type guard failure.
12. Compare representative pre-slice IL. Direct calls and declarations that are never used as values must retain their direct-symbol form and existing ABI. Record descriptors, heap headers, and allocation behavior from Slice 15 must remain unchanged.
13. Search for `not yet supported: PROCEDURE types`, stale procedure-as-value diagnostics, and remaining correct-source paths to unsupported handling. Correct source using every grammar production must not reach an unsupported diagnostic, panic, QBE error, assembler error, or linker error.

## What remains unsupported

No May 2016 core grammar production remains deliberately unsupported after this slice. Portable standard modules, the conformance matrix, malformed-source closure, and the complete release gate remain Slices 17 and 18. The optional `SYSTEM` module remains outside core completion.

Closures, nested-procedure values, foreign function pointers, variadic procedures, call chaining, reflection over signatures, user-visible code addresses, and conversions between data pointers and procedure values are not Oberon-07 features and are not introduced.

## Order of work

1. Add procedure type syntax to the AST and parser, share the formal-parameter parser with procedure declarations, and pin parser shapes and malformed syntax.
2. Add semantic procedure signatures, recursive exact equality, storage rules, display, result restrictions, and centralized compatibility in `src/sema/types.rs`.
3. Make declared and exported procedures carry that semantic type plus direct-symbol and eligibility information. Keep nested and predefined direct calls working while rejecting them as values.
4. Add procedure values to assignment, relations, value and `VAR` parameter substitution, aggregate storage, and function returns. Exercise each role before adding indirect activation.
5. Generalize the typed IR call target and QBE emitter for direct symbols and indirect values. Add the dedicated zero-procedure runtime check.
6. Refactor call lowering to resolve and capture a callable before building arguments, then route direct and indirect calls through the existing argument expansion. Verify scalar, structured, open-array, record-descriptor, and procedure arguments independently.
7. Complete constant-expression type walks and the Slice 15 terminal application decision for procedure-valued places.
8. Add the positive, negative, runtime-failure, cross-module, aggregate, ABI, and evaluation-order gates. Add a regression module for every bug found during implementation.
9. Update architecture documentation, run the full verification list, inspect the complete diff, and report the slice ready for independent review.
