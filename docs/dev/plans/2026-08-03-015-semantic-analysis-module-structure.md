# Slice: Semantic analysis module structure

## Context

`src/sema.rs` is 5,326 lines and contains several independently coherent parts of semantic analysis. It defines the semantic type and symbol models, checks and evaluates constant expressions, and lowers executable declarations, statements, expressions, designators, and calls into typed IR. The crate-facing surface is small: the driver uses `analyze`, `Interface`, and `out_interface`.

This slice separates the type model, symbol and interface model, and constant-expression analysis into child modules under `src/sema/`. The remaining AST-to-IR analyzer stays together in `src/sema/mod.rs`. The filesystem change preserves `crate::sema` as the pipeline-stage boundary and preserves the driver's existing paths.

This is a structural refactor. It changes no Oberon-07 rule, diagnostic text or ordering, semantic representation, IR, generated code, runtime behavior, or dependency.

## Module boundaries

### `src/sema/types.rs`

This module owns `Type`, `ArrayType`, `RecordType`, `PointerType`, their supporting descriptors, type identity and display, storage layout queries, field visibility lookup, assignment and parameter compatibility, operand type checking, and array-shape derivation from semantic types.

Helpers that describe emitted operations do not move here merely because they inspect a type. In particular, the BYTE runtime-domain check and relation-to-IR mapping remain with executable lowering. Constant-value comparison remains with constant analysis.

### `src/sema/symbols.rs`

This module owns `ConstValue`, the internal `Symbol` model, the exported `Interface` and `Member` model, predefined-operation identities and signatures, universe-scope construction, conversion of an exported member into a client symbol, and the temporary native `Out` interface.

`ConstValue` lives here because constants are stored in scopes and module interfaces. Constant-expression analysis consumes this representation but does not own the exported value model.

### `src/sema/constant.rs`

This module owns the `Analyzer` methods that type-check, classify, and evaluate constant expressions. It includes the non-emitting designator walks used by ordinary constant expressions and required-constant `LEN`, plus constant-only arithmetic, text-comparison, set, and `FLOOR` helpers.

The constant path remains a semantic subpass over the existing AST and scope state. It does not gain a visitor abstraction, a separate context object, or duplicated symbol lookup.

### `src/sema/mod.rs`

This module owns `Analyzer`, its mutable state, executable declaration and AST-to-IR lowering, places and sources, procedure construction, shared emission plumbing, and lowering-specific diagnostics and helpers. It declares the three child modules and re-exports the crate-facing names needed by the driver.

The executable lowering is deliberately not split into declaration, statement, expression, designator, and call siblings in this slice. Those methods call one another heavily, and the three extracted concerns are sufficient to leave one coherent lowering walk behind.

## Rust visibility

Private fields declared on `Analyzer` remain accessible to its descendant modules. Associated methods and type members declared inside a child module do not become accessible to the parent or sibling modules automatically. Cross-module implementation details therefore use `pub(super)` where required. This exposes them only within `sema` and does not enlarge the crate-facing API.

`crate::sema::analyze`, `crate::sema::Interface`, and `crate::sema::out_interface` retain their existing paths. `src/driver.rs` and `src/main.rs` remain unchanged unless the compiler demonstrates that an explicit import adjustment is required.

## Mechanical constraints

Code and comments move without semantic rewriting. Existing comments stay with the behavior they explain. Visibility changes are limited to what the new module boundaries require. No compatibility helper is duplicated to avoid a module reference, and no wrapper abstraction is introduced solely to conceal the split.

The extraction proceeds one module at a time. Each step must compile before the next boundary is introduced, which makes missing imports and insufficient visibility local and reviewable.

## Files changed

- Replace `src/sema.rs` with `src/sema/mod.rs`.
- Add `src/sema/types.rs` for semantic types and compatibility.
- Add `src/sema/symbols.rs` for scopes, symbols, interfaces, constant values, and predefined-operation metadata.
- Add `src/sema/constant.rs` for constant-expression checking and evaluation.
- Update `docs/dev/code-style.md` so its structural rule names pipeline modules and the cohesive semantic-analysis submodules accurately.
- Keep `src/driver.rs`, `src/main.rs`, AST, IR, backend, runtime, corpus, and architecture documentation behaviorally unchanged.

## Verification

1. Run `cargo fmt --check`.
2. Run `cargo clippy --all-targets -- -D warnings`.
3. Run `cargo test` to exercise the complete positive, diagnostic, runtime-failure, and cross-module corpus.
4. Run `git diff --check`.
5. Compare the public uses of `crate::sema` before and after the extraction and confirm the driver still names only `analyze`, `Interface`, and `out_interface`.
6. Search the new modules for duplicated definitions and confirm every moved function has one owner.
7. Inspect the diff with move detection to confirm the slice consists of relocation, module declarations, imports, and the minimum `pub(super)` visibility changes.

## Declined work

This slice does not change semantic behavior, add unit tests for existing compatibility rules, redesign diagnostics, alter exported interface visibility, or introduce new abstractions. It also does not split the remaining executable analyzer by Oberon grammar category. Those decisions require evidence from the extracted result rather than being bundled into this mechanical refactor.

## Order of work

1. Create `src/sema/mod.rs` from the current semantic analyzer and establish the child-module declarations.
2. Extract the constant-expression subpass and its constant-only helpers, then compile and resolve only the visibility and imports required by that boundary.
3. Extract the semantic type model, compatibility rules, and type-derived shape helpers, then compile and resolve its boundary.
4. Extract symbols, interfaces, predefined-operation metadata, and the native `Out` interface, then restore the existing `crate::sema` surface.
5. Format, lint, run the complete test suite, inspect the diff, and report the slice ready for independent review.
