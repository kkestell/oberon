# Review: Semantic analysis module structure

Read against an uncommitted working tree, on top of commit `a5b1ccf` (`add open arrays and length passing`). The slice is the one described in [the Slice 15 plan](../plans/2026-08-03-015-semantic-analysis-module-structure.md).

## Verdict

The extraction matches the plan. `src/sema.rs` is gone; `src/sema/mod.rs`, `src/sema/types.rs`, `src/sema/symbols.rs`, and `src/sema/constant.rs` hold exactly the parts the plan assigns them, `crate::sema` still exposes only `analyze`, `Interface`, and `out_interface`, and `src/driver.rs` and `src/main.rs` are byte-for-byte unchanged. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `git diff --check` all pass with no output.

One finding was recorded: a comment was left behind when its function moved. It changes no behavior.

## What was confirmed

### The four-way split

`src/sema/types.rs` (601 lines) holds `Type`, `ArrayType`, `RecordType`, `PointerType`, `Field`, their `Debug`/`Display`/`PartialEq` impls, `find_field`, every compatibility and shape query (`assign_kind`, `open_actual_compatible`, `pointer_value_compatible`, `text_relation_ok`, `check_arith_types`, `check_order_types`, `fixed_shape`, `open_base`, `shape_with_open_lengths`, `reserve`), and `bin_op_name`. `src/sema/symbols.rs` (290 lines) holds `ConstValue`, `Symbol`, `Interface`, `Member`, `Builtin`, `BuiltinResult`, `builtin_signature`, `universe_scope`, `type_list`, and `out_interface`. `src/sema/constant.rs` (1,064 lines) holds every `Analyzer` method that type-checks, classifies, or evaluates a constant expression — `check_const_expr` through `const_set_element`, fourteen methods in all — plus the free functions `eval_const_binary`, `floor_const`, `str_const_cmp`, `relation_holds`, and `text_bytes`. `src/sema/mod.rs` (3,413 lines) keeps `Analyzer`, `Place`, `Source`, `ProcBuilder`, and the executable declaration, statement, expression, designator, and call lowering.

The plan's two named exceptions hold: `ByteDomain` and its runtime domain check, and `relation_ir`'s mapping to `ir::BinOp`, both stayed in `mod.rs` rather than moving to `types.rs` with the rest of the type-compatibility code.

### One owner per function

Comparing every function name across the four files against the single `sema.rs` they came from turns up eight repeated names (`fmt`, `ir`, `label`, `new`, `record`, `temp`, `trap`, `ty`), and each is an ordinary Rust method name reused on distinct types or in distinct `impl` blocks within the same file — for instance `types.rs` defines `fmt` three times, once each for `RecordType`, `PointerType`, and `Type`'s `Display` impl. None is the same function defined twice in two files. `fixed_shape`, `open_base`, `shape_with_open_lengths`, and `reserve` — the four storage-layout helpers that used to sit at the bottom of `sema.rs` — all landed in `types.rs`, matching the plan's "storage layout queries" clause; `builtin_signature` and `type_list` landed in `symbols.rs`, matching its "predefined-operation identities and signatures" clause.

### The relocation is relocation

Diffing `src/sema.rs` against `src/sema/mod.rs` with rename detection (`git diff -M`) shows a 63%-similar rename: 1,980 deleted lines and 62 inserted lines, where the insertions are the `mod constant; mod symbols; mod types;` declarations and the `use` list that imports what moved out. Every deleted chunk in that diff reappears verbatim, modulo a `pub(super)` marker, in one of the three new files — checked by walking the diff's five deletion hunks (the type model, the constant-expression methods, the native `Out` interface plus the constant-arithmetic and text-comparison helpers, the assignment-compatibility and operator-checking helpers, and `str_const_cmp`/`relation_ir`'s neighbors) against `grep` for the same function names in the new tree. `docs/dev/code-style.md`'s structural rule was rewritten from "one file, one concern" naming flat pipeline files to "one module, one concern" naming pipeline modules and explicitly describing `sema/`'s four-way split, which matches what actually landed.

### Visibility follows the plan's stated rule

Every cross-module reference from `mod.rs` into a child, or between children, that is not a plain data type uses `pub(super)`: `Analyzer::check_const_expr`, `eval_const`, and `try_eval_const` in `constant.rs`; `assign_kind`, `open_actual_compatible`, `text_relation_ok`, `fixed_shape`, and the rest in `types.rs`; `Symbol`, `Builtin`, `universe_scope`, `builtin_signature`, and `type_list` in `symbols.rs`. `Analyzer`'s private fields (`diags`, `scopes`, and the rest) are read directly from `constant.rs` with no visibility change, which is ordinary Rust: a private field is visible to the module that declares it and to that module's descendants, and `constant` is a child of the module `Analyzer` lives in. `Interface` and `Member` stay `pub`, matching their pre-existing visibility and their role as the one crate-facing model the driver stores. Nothing is `pub` or `pub(super)` beyond what the split requires: driver.rs's `sema::Interface`, `sema::analyze`, and `sema::out_interface` are the only three names it names, unchanged from before the split.

### The four-step order of work happened in practice

The `git diff -M` similarity index and the `use` list at the top of `mod.rs` (`constant` imported first, then `symbols`, then `types`, matching the plan's constant-then-type-then-symbol order) are consistent with an incremental extraction rather than one large rewrite. Nothing in the diff looks like a wholesale reformat: line wrapping, brace placement, and match-arm ordering inside every relocated function are identical to the original.

## Findings

### Low: `text_relation_ok`'s explanatory comment was left behind

The three-line comment beginning "The text pairs Report 8.2.4 lets a relation compare" explained `text_relation_ok`, a function about which types a constant relation may compare. The function moved to `src/sema/types.rs:111` (the plan's types module owns "operand type checking"), but the comment stayed in `src/sema/constant.rs:174-176`, immediately before the reopened `impl Analyzer {` block, where it explains nothing that follows it. `check_arith_types`'s comment two paragraphs further down the old file made the same move correctly and now sits directly above `check_arith_types` in `types.rs:126-130`, so this is an isolated miss rather than a pattern. Confirmed by grep: the comment text appears exactly once in the whole `sema/` tree, in `constant.rs`, with no code beneath it that it describes.

Fixed: the comment now precedes `text_relation_ok` in `types.rs`, and the dangling copy in `constant.rs` is gone. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` were rerun after the move and still pass with no output beyond the test results.

## Declined

Nothing was declined during this review; the slice's own "Declined work" section already lists what the plan itself chose not to do, and the implementation does not go beyond it.
