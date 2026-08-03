# Review: Fixed arrays and checked indexing

Reviewed against the uncommitted working tree on top of commit `682c16c` (`real values and numeric conversions`). The implementation follows [the Slice 10 plan](../plans/2026-08-02-010-fixed-arrays-and-checked-indexing.md).

The untracked Slice 11 plan is outside this review. It contains no implementation changes that affect the Slice 10 result.

## Verdict

The core array implementation is faithful to the plan. Type identity, fixed layout, exported type handles, checked nested indexing, selected scalar variables, whole-array copying, fixed-array `LEN`, storage limits, and QBE lowering all work on the supplied positive, diagnostic, and runtime-failure paths.

Two medium-priority findings were confirmed and fixed. No findings remain.

## Findings

### Constant `LEN` selectors can panic before they are type-checked

`Analyzer::check_const_len` reaches `Analyzer::designator_type` through `const_array_type`. `designator_type` sends each index expression directly to `eval_const`. The constant evaluator assumes semantic type checking has already succeeded and uses `unreachable!` for impossible operand combinations.

That assumption is false on this path. Unlike ordinary executable indexing, the required-constant `LEN` path never checks a selector expression before evaluating it. A composite selector with invalid operand types therefore crashes the compiler instead of producing a source diagnostic.

This valid-syntax program confirmed the problem:

```oberon
MODULE LenBadConstIndex;
  VAR a: ARRAY 2 OF ARRAY 3 OF INTEGER;
  PROCEDURE P;
    CONST N = LEN(a[1 + TRUE]);
  END P;
END LenBadConstIndex.
```

Running `target/debug/oberon /tmp/LenBadConstIndex.Mod` exited with status 101 and wrote:

```text
thread 'main' panicked at src/sema.rs:3290:17:
internal error: entered unreachable code: constant expression was type-checked
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
```

The process-specific identifier printed after `main` has been omitted from the transcript. The compiler should instead report that `+` requires operands of one accepted type and reject the source normally. The same unchecked path can misdiagnose a non-INTEGER variable selector as merely nonconstant.

The non-emitting designator check used by constant `LEN` needs to type-check every selector expression before folding it. A regression should put an ill-typed composite selector inside a `LEN` used by a constant declaration or array length and assert a source diagnostic with no panic.

#### Resolution

`Analyzer::check_const_designator_type` now type-checks every selector before asking the constant evaluator for its value. Required-constant designators and `LEN` both use this checked path, while evaluation retains the smaller invariant-based path after checking has succeeded.

`ArrayLenConstIndexBad.Mod` covers a mixed-type composite expression and a BOOLEAN variable selector inside local constant `LEN` calls. The compiler exits unsuccessfully without panicking and writes:

```text
tests/errors/ArrayLenConstIndexBad.Mod:6:27: operator '+' requires two INTEGER, two REAL, or two SET operands, found INTEGER and BOOLEAN
tests/errors/ArrayLenConstIndexBad.Mod:7:25: array index must be INTEGER, found BOOLEAN
Error: 2 error(s)
```

### Declaration sections are reordered instead of enforcing textual order

The Report permits at most one `CONST`, one `TYPE`, and one `VAR` section, in that order. The Slice 10 plan also states that declarations obey textual scope. `Parser::declarations` instead loops over all three section keywords in any order and appends their contents to separate vectors. `Analyzer::module` and `Analyzer::procedure` then analyze every constant, every type, and every variable in phase order, regardless of their source order.

This means a type written after a variable can become visible to that earlier variable. The following source is outside the Report grammar because its `TYPE` section follows `VAR`, but the compiler accepts it:

```oberon
MODULE ArrayDeclarationOrder;
  IMPORT Out;
  VAR a: Later;
  TYPE Later = ARRAY 1 OF INTEGER;
BEGIN
  a[0] := 7;
  Out.Int(a[0], 0); Out.Ln
END ArrayDeclarationOrder.
```

Running `target/debug/oberon /tmp/ArrayDeclarationOrder.Mod` exited with status zero and wrote nothing. Running the generated `build/ArrayDeclarationOrder` also exited with status zero, wrote no standard error, and printed:

```text
7
```

The parser should enforce the declaration sequence directly. Once it has passed a section, seeing that section or an earlier one again should be a parse error rather than source that semantic analysis silently reorders. A regression should cover at least `TYPE` after `VAR`; repeated and otherwise interleaved sections use the same faulty loop.

#### Resolution

`Parser::declarations` now parses `CONST`, `TYPE`, and `VAR` in three ordered optional phases. A repeated or earlier section keyword remains for the enclosing module or procedure production, which reports the syntax error instead of building a reordered AST.

The parser unit test pins the section-order rule. The end-to-end `ArrayDeclarationOrder.Mod` regression exits unsuccessfully and writes:

```text
tests/errors/ArrayDeclarationOrder.Mod:3:3: expected 'END', found Type
Error: 1 error(s)
```

## Verified implementation

Each source `ARRAY` constructor receives shared semantic descriptors. Aliases and cloned interfaces preserve descriptor identity, while separately written constructors remain distinct even when their printed shapes match. The cross-module program confirms that an original exported name and a re-exported alias remain assignment-compatible.

Array layout is recursive and contiguous. A `2` by `3` INTEGER array reserves 24 bytes, its row stride is 12, and current array alignments are four. Zero-length globals and locals reserve zero payload bytes and still support `LEN`, checked selection, and zero-byte assignment.

Every executed selector lowers to an IR index instruction carrying its applicable length and stride. QBE calls `oberon_check_index` before extending, scaling, or adding the index. The first-dimension failure IL places the first check before any instruction that evaluates the second index expression.

Selected INTEGER, BOOLEAN, and SET elements use word loads and stores. Selected REAL elements use single loads and stores. Selected writable scalar elements work through scalar `VAR` parameters and all applicable modifying predefined procedures.

Whole-array and selected-row assignments call `oberon_copy` with the exact byte count. Destination selectors precede source selectors and each occurs once. Self-assignment remains present, and zero-length copies still resolve both designators before calling the runtime with a zero count.

Exported array variables with private or inline types remain readable and indexable in clients without exposing a type name. Imported arrays and all selected parts retain read-only status.

The per-type, procedure, module, and whole-program storage limits all reject the supplied oversized cases before QBE or the linker reports a target failure. Fixed-array parameters stop at the planned unsupported diagnostic, and array results receive the permanent Report 10.1 diagnostic.

## Declined changes

The review does not request structural array equivalence. Descriptor identity is the deliberate rule in the plan and is preserved across declarations and module interfaces.

The review does not request scalarizing array copies or representing arrays as pointer values. The explicit address-to-address runtime copy correctly preserves value semantics and handles self-assignment.

The review does not request an optimization that removes checks for valid constant indices in executable code. Retaining one checked execution path is a deliberate plan decision.

The review does not cover CHAR, BYTE, strings, structured parameters, records, pointers, or open arrays. Those remain assigned to later slices.

## Verification

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `git diff --check` pass. The test run contains 28 unit tests and two integration tests.

`Arrays`, `ArrayElements`, `ArrayDimensions`, `ArrayCopy`, and `ArrayApi` were also compiled and run directly. Every compiler and program invocation exited with status zero, every standard error stream was empty, and every standard output matched its checked-in expectation.

The generated IL for `ArrayDimensions`, `ArrayElements`, `ArrayCopy`, all five array failure programs, and `ArrayApi` was inspected for allocation size, alignment, scalar load class, check order, dimension length, stride, copy size, selector order, zero-size behavior, and cross-module symbol use.

The unchanged `Params` module still produces SHA-256 `bc6627e4199e252930401829769ea56477ae76987738f442f8111d8121233291`. This is the hash recorded before the array refactor and confirms that the representative scalar IL remained byte-for-byte stable.
