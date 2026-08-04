# Review: Procedure types and indirect calls

Written against the uncommitted tree implementing [the procedure-types-and-indirect-calls plan](../plans/2026-08-04-017-procedure-types-and-indirect-calls.md), based on commit `cbe0276` (record extension and dynamic type operations).

## Gate

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` all pass clean: 42 unit tests and the two integration tests (`corpus`, `root_filename_must_end_in_mod`), which sweep every `.Mod`/`.expected` pair under `tests/corpus`, `tests/errors`, and `tests/failures`, including every module the plan lists as new. A repo-wide search for `not yet supported: PROCEDURE types` and other stale procedure-as-value diagnostics turned up nothing; the `unsupported` parser helper the old rejection used is gone entirely.

## What was checked

Every changed file (`ast.rs`, `parser.rs`, `ir.rs`, `qbe.rs`, `sema/types.rs`, `sema/symbols.rs`, `sema/mod.rs`, `sema/constant.rs`, `runtime/oberon.c`, `docs/dev/architecture.md`) was read in full against its diff, not just skimmed. The read focused on the plan's specific claims: recursive structural matching for value contexts, constructor-identity matching for `VAR` actuals and function results, module-scope-only eligibility, shared direct/indirect argument construction, and the callee-before-actuals evaluation order.

**Signature matching.** `procedure_signatures_match` in `src/sema/types.rs` compares parameter count, mode, and type positionally, recursing into `signature_component_match` for open-array and procedure-typed components while falling back to `Type`'s `PartialEq` (identity via `Rc::ptr_eq`) for everything else, including the function result type. This gives exactly the asymmetry the plan calls for: two independently declared procedure types with matching shapes are assignable to each other, but the same two types used as a function result, or as a `VAR` formal's expected type (checked separately with a plain `==`), are not. `tests/errors/ProcedureTypeBad.Mod` exercises this asymmetry directly — `outerA := outerB` (parameter types `InnerA`/`InnerB`, structurally equal, different constructors) is accepted, while `returnA := returnB` (same two types as function results) is rejected — and the `.expected` file confirms the compiler agrees with both halves.

**Eligibility.** `Symbol::Proc` carries an `eligible` flag set from `self.scopes.len() == MODULE_SCOPE + 1` at the declaration site, so a nested procedure is never eligible and a module-level one always is. Imported members reach `Symbol::Proc` through `Member::to_symbol`, which hardcodes `eligible: true` — correct, since only exported (necessarily module-scope) procedures cross module boundaries, and the `Out` runtime members are built the same way, making `Out.Char` usable as a value while `ABS`/`INC`/`LEN` (still `Symbol::Builtin`) are not. `tests/corpus/modules/procedure-api/ProcedureApi.Mod` exercises `action := Out.Char` directly.

**Evaluation order.** This is the subtlest claim in the plan, so it was checked against the actual emitted IL rather than just the source. `tests/corpus/ProcedureCallOrder.Mod` indexes into an array with a side-effecting index expression, then passes a side-effecting actual that mutates the same array slot before the call fires. The built `build/ProcedureCallOrder.ssa` shows:

```
%.t1 =w call $ProcedureCallOrder.TargetIndex()
call $oberon_check_index(w %.t1, w 1)
%.t3 =l loadl %.t2      ; captures table[TargetIndex()] into %.t3 before the actual runs
%.t4 =w call $ProcedureCallOrder.Change()   ; actual now overwrites table[0]
call $oberon_check_procedure(l %.t3)
call %.t3(w %.t4)       ; the captured value, not the post-mutation one, is invoked
```

which is exactly the order the plan specifies, and the program's actual output (`taAB`) matches `ProcedureCallOrder.expected` and confirms it at runtime, not just in the IL shape. `tests/failures/NilProcedureCallOrder.Mod` confirms the mirror case: the actual with a visible side effect still runs before the dedicated nil check fires.

**Shared call machinery.** `lower_call` in `src/sema/mod.rs` resolves a direct symbol or an indirect value (loading the latter once, before the argument loop) into one `(CallTarget, Type, Option<Value>)` triple, then runs the same argument-construction loop regardless of which target it is. `ir::Inst::Call` and the QBE emitter both take a `CallTarget` enum rather than a bare symbol string, so a direct call still emits `call $symbol(...)` and an indirect one emits `call %temp(...)` with identical argument and result classes. `tests/corpus/ProcedureOpenArrays.Mod` calls the same open-array and record-`VAR` callback both directly and indirectly and compares output byte for byte, which is the strongest available proof that the hidden-argument expansion (open-array lengths, record descriptors) is unaffected by the target kind.

**Constant-expression walk.** `src/sema/constant.rs` was checked for the same asymmetry the plan requires: a procedure designator, comparison, or call is well-typed (so `check_const_expr` reports the right diagnostic — e.g. "is not a procedure", not a generic constant-folding failure) but never reaches `eval_const`'s folding cases; `eval_const`'s `Apply` arm only fires for the already-typechecked, already-diagnosed path, so its collapsing of every non-builtin resolution into "constant expression contains a procedure call" does not discard a more specific diagnostic — `const_declarations` only calls `eval_const` after `check_const_expr` has already succeeded on the same expression. `tests/errors/ProcedureConstBad.Mod` confirms all three constant-context rejections (procedure name, procedure relation, procedure call) with distinct, specific messages.

**Heap layout.** `Type::contains_pointers` was not given a new arm for `Type::Procedure` — it falls through the wildcard to `false` — which is correct and is exactly what keeps a procedure field from forcing scanned allocation. `tests/corpus/ProcedureAggregates.Mod` stores procedures in arrays, records, and a pointer-based record, including through a guard selector on an extended record, but does not itself assert atomic allocation; that property rests on the existing wildcard match rather than new-code coverage, which is fine given how small and clearly-in-scope the omission is.

**Diagnostic text changes on unrelated existing tests.** `tests/errors/ProcAsValue.expected`, `TypeGuardBad.expected`, and `TypeTestBad.expected` changed because their fixtures reference a procedure name directly (`P`) in a context (assignment, type guard, type test) that previously had no value semantics to describe and now does. Each new message was checked against its trigger: `x := P` now reports the ordinary `cannot assign PROCEDURE to INTEGER` mismatch instead of a fallback "cannot be used as a value"; `n(PChild)` where `n: INTEGER` now reports `'n' has type INTEGER and is not a procedure` because `application_kind` classifies an `INTEGER` prefix as neither a callable nor a guardable place and routes it through the call diagnostic rather than the guard-specific one; `P IS PChild` now reports the type-test diagnostic with `found PROCEDURE` instead of the old fallback. All three are consequences of procedures having a real semantic type now, not regressions.

## Issues found

None. Every claim in the plan that was checked against the diff, the test fixtures, and the emitted IL held up, and the full gate is green. No changes were made as part of this review.
