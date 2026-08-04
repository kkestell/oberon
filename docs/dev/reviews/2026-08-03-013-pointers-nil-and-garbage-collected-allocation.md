# Review: Pointers, NIL, and garbage-collected allocation

Read against an uncommitted working tree on top of commit `0a54908` (`review records and structured parameters`). The slice is the one described in [the Slice 13 plan](../plans/2026-08-02-013-pointers-nil-and-garbage-collected-allocation.md).

## Verdict

The pointer representation, the forward-reference rule, checked dereference, the allocation-kind classification, and the collector integration are all correct, and every one of them survived probing. Cross-module identity, read-only propagation, and the eight-byte layout behave as the plan describes.

One behavioural defect was found, and it is not in the pointer code. Rewriting the required-constant `LEN` walk so that it stops evaluating index expressions also stopped it rejecting a procedure call in an index, so a constant declaration could name a call that the compiler silently never performed. The remaining findings were missing tests for behaviour that works, diagnostics printing out of source order, and two places where the prose was out of step with the code.

Every finding was fixed. The defect is fixed in `src/sema.rs`, the ordering in `src/driver.rs`, and the rest in the corpus, the runtime, and the two documents. The gate is green after all of it, and code generation for pointer-free programs is still byte-identical to the pre-slice compiler.

## Findings

### Medium: a required-constant `LEN` accepts a procedure call in an index and never calls it

This program compiles, prints `3` for the constant, and prints `0` for the call counter:

```oberon
MODULE ProbeQ;
IMPORT Out;
VAR a: ARRAY 2 OF ARRAY 3 OF INTEGER; calls: INTEGER;
PROCEDURE Index(): INTEGER;
BEGIN
  INC(calls)
RETURN 0
END Index;
PROCEDURE Fold;
CONST A = LEN(a[Index() + 1]);
BEGIN
  Out.Int(A, 0); Out.Int(calls, 0)
END Fold;
BEGIN
  Fold; Out.Ln
END ProbeQ.
```

Its output is `30`, meaning the constant folded to three and `Index` never ran. The same happens when the folded length is an array bound, so `VAR v: ARRAY LEN(a[Index()]) OF INTEGER` also compiles with the call discarded. The defect has nothing to do with pointers: it reproduces on a plain array and on a record field.

Compiled from commit `0a54908` — that is, the same source before this slice — the first form is rejected with `constant expression contains a procedure call` at the position of the call. So this slice removed a diagnostic that existed and was right.

The cause is in `Analyzer::check_len_designator_type` and in the `LEN` arm of `Analyzer::is_const_expr`. The checking walk type-checks each index with `check_const_expr`, which happily returns a user procedure's result type, and then asks `try_eval_const` for the value. `try_eval_const` answers `Ok(None)` for anything it cannot fold, which is how a variable index is meant to pass, and a call reaches that same answer. Meanwhile `is_const_expr` delegates to `len_designator_type`, which skips index expressions entirely, so the whole `LEN` call is classified as constant no matter what is inside the brackets.

The plan's licence was narrower than this. It says the walk "accepts a dynamic INTEGER index", and the reference behaviour it cites is a *variable* index. A variable read that is skipped is unobservable. A call that is skipped is not, and Report 8's rule that a constant expression is evaluable by a textual scan without executing the program is exactly the rule against it.

Fixed. A new `Analyzer::const_index_call` walks an index expression looking for a call to anything that is not a predefined operation, and returns the existing `constant expression contains a procedure call` diagnostic when it finds one. It descends into set elements, unary and binary operands, the arguments of a predefined call, and the index selectors of any designator it passes, so `LEN(a[LEN(b[F()])])` is caught as well as `LEN(a[F()])`. Both walks consult it — `check_len_designator_type` reports the diagnostic, `len_designator_type` returns it — because they have to reach the same verdict or a constant declaration would be checked under one rule and evaluated under another. `ProbeQ` above is now rejected at the position of the call.

A variable index still folds, which was the point of the rewrite. A probe declaring `CONST A = LEN(a[k]); B = LEN(p.row[k]); C = LEN(p^.row[k]); D = LEN(a[1])` and a local `ARRAY LEN(a[k]) OF INTEGER` printed `33333`.

`tests/errors/PointerLenBad.Mod` now pins the rule in all three contexts that reach the walk: a call through a pointer selector, a call inside arithmetic in a plain array index, and a call in an array-length position. It reports six diagnostics and no cascades.

[docs/dev/architecture.md](../architecture.md) had asserted the opposite of what the code did. Its required-constant `LEN` paragraph ended "Nothing observable is skipped because this path asks only for the fixed array type and emits no executable expression." A dropped call is observable. The paragraph now says which two things the path declines to do and why each is unobservable — a variable read has no effect, and a call does, so a call is an error.

### Low: forward-base diagnostics print out of source order

`tests/errors/PointerForwardBad.expected` pins its diagnostics in the order line 7, line 8, line 4, line 5, line 6, line 12. Pending pointer bases are resolved after the whole `TYPE` section has been processed, so their diagnostics are appended after every diagnostic that later declarations in the same section produced.

This was the only `.expected` file in `tests/errors` whose line numbers were not increasing; a scan of all of them found no other. So a reader who had learned that this compiler lists errors top to bottom was contradicted for the first time here, on a file whose errors are already the subtlest kind in the slice.

Fixed at the class rather than the instance. `driver::fail` now sorts diagnostics by line and column before printing them. The sort is stable, so two diagnostics at one position keep the order the analyzer reported them in, and a pass that wants a particular order among them still gets it. Deferring a check is now free of presentation consequences, which is what makes resolving a pointer's forward base after its `TYPE` section a clean design rather than a visible one.

Sorting changed exactly one expected file, `PointerForwardBad.expected`, which now runs 4, 5, 6, 7, 8, 12. Every other error expectation was already in order and is untouched. [docs/dev/code-style.md](../code-style.md) records the guarantee alongside the diagnostic struct it belongs to.

### Low: behaviour the plan promised a test for, which worked but was untested

Each item below was confirmed working by compiling and running a probe, and none of them was covered by a committed test even though the plan's corpus section says each one would be. All six are now in the corpus, and the two modules that gained them still produce their allocation kinds and check placement unchanged.

Whole-record assignment through pointers was missing from `Pointers.Mod`. It now runs `record := head^`, mutates the copy, writes it back with `head^ := record`, and prints the value before and after, giving `41` then `42`. The IL puts `oberon_copy` on the checked heap address on the correct side each time.

A pointer type alias was missing. `Pointers.Mod` aliased an array of pointers, not a pointer. It now declares `Alias = P`, assigns `head` across the alias, reads through `aliased^`, and passes the aliased variable to `Clear`, whose formal is `VAR p: P`. That last call is the one that fails if an alias does not share the pointer descriptor, because a `VAR` formal demands identity.

Executable `LEN` through a pointer had only its nil-failure form, in `tests/failures/NilDerefLen.Mod`. `Pointers.Mod` now evaluates `LEN(head.rows)` and `LEN(head^.rows[dynamicIndex])` in statement position on an allocated pointer and prints `2` and `3`, beside the constant declarations in `Constants` that fold the same two lengths.

A folded `NIL` equality was promised beside the runtime comparison and was not there. `IF NIL = NIL THEN` is not folded — `build/Pointers.ssa` emits `ceql 0, 0` for it — because folding only happens where a constant is required. `Pointers.Mod` now declares `CONST NilFolds = (NIL = NIL)` and branches on it immediately after the runtime form, so the two sit next to each other as the plan intended.

`PointerGc.Mod` was to retain its list "solely through generated pointer variables and fields", but during the 200000-allocation churn loop the list was held by the module variable `head`, and `holder.nested.links[0] := head` ran only after the loop. It now hands the list to that field *before* the churn and clears `head` and `temporary`, so from there to the traversal the whole chain is reachable only through a pointer inside an array field inside a record field of a heap object. The traversal afterwards both counts the nodes and sums their values, which catches a chain that survived partially as well as one that did not survive at all. It prints `1000` and `500500`, and under `GC_PRINT_STATS=1` the run reports 42 collections. This is now the test that would fail if `Holder` were ever classified atomic.

`Pointers.Mod` was to read its array of pointers back "after allocation pressure", and nothing allocated between `Fill`, the whole-array copy, and `Read`. A 100000-iteration allocation loop now sits between the copy and the read, and `Read` still prints `2` and `3`.

### Low: the two new lines in `runtime/oberon.c` carry no comment

Every other helper in that file cites the Report rule it implements — `oberon_check_index` explains why the check precedes the address calculation, `oberon_copy` explains why it is `memmove`. The two additions do not follow suit.

`oberon_check_nil` would survive without one, though it now cites Report 8.1 like its neighbours. `GC_set_all_interior_pointers(1)` would not. It was the least obvious line in the file, and its reason is specific: generated code can hold an address *into* a heap object while nothing points at that object's first byte, so the object has to stay alive anyway. That reasoning existed only in the plan, which is not where a future reader looks.

The hazard is real, incidentally, not merely defensive. `p.next := F()` computes the address of `p`'s `next` field, then calls `F`, which may allocate and therefore may collect. If `F` also clears `p`, the only remaining reference to the old node is that interior address. On this machine the installed BDWGC 8.2.12 already defaults `GC_all_interior_pointers` to one, which a small C probe confirmed, so the call changes nothing here; the point of making it explicit is that a differently built collector would not.

Fixed. The comment now gives that example, says the setting only takes effect before the collector initializes and therefore why the two calls cannot be swapped, and says the setting is requested rather than assumed because whether it is on by default is a property of the build.

## Verified implementation

### Representation and layout

`PointerLayout.Mod` fixes the eight-byte layout end to end. Its record places `ch: CHAR` at offset zero, `first` at eight, `byte: BYTE` at sixteen, and `items: ARRAY 3 OF ItemPointer` at twenty-four, for a total size of forty-eight — the padding a four-byte pointer would not produce. `build/PointerLayout.ssa` emits `data $PointerLayout.layout = align 8 { z 8 }` for the global, strides the array by eight, and uses `loadl` and `storel` for every pointer access. Pointer locals emit `alloc8 8`, visible in `Pointers.LocalTypes`. The program prints `A25578910`.

`build/Pointers.ssa` confirms the parameter and result conventions. `Same` is `function l $Pointers.Same(l %.t0)` and copies its incoming pointer into its own `alloc8 8` slot before returning it, so reassigning a pointer value parameter cannot reach the actual — which `ValueCopy` then demonstrates, since `head.value` still prints `41` after the call. `Clear` receives `l $Pointers.head`, the address of the caller's pointer storage, and stores zero through it. Assignment between the two distinct constructors `P` and `Q` that share the base `Node` emits a plain `loadl`/`storel` pair with no conversion.

### Dereference

Every dereference loads once, checks once, and only then uses the value as an address. `build/NilDerefChain.ssa` shows the chained case in full: `p` is loaded, checked, and offset to reach `next`; `next` is loaded, checked, and offset to reach `n`. Two checks, in source order, one load each. `p^.field` produces a single check. Explicit and implicit forms cannot diverge because `Analyzer::field` routes a pointer base through `Analyzer::dereference` rather than reimplementing it.

The four runtime-failure modules each exit nonzero with exactly `nil pointer dereference` on standard error. `LinkedList.Mod` walks its list under `(head # NIL) & (head^.value > 0)` and prints `3210`, so the short-circuit guard keeps the dereference off the null path.

Read-only status follows the address through a dereference, which is the behaviour Project Oberon has: `ORG.DeRef` in the vendored copy does not touch `x.rdo`, and `ORP.selector` calls it for both `^` and an implicit field selection. A probe passing a record by value confirmed all four consequences — `v.p := NIL`, `v.p.n := 1`, `NEW(v.p)`, and `v.p^.n := 2` are each rejected, the last of them naming the designator as `v.p^.n`, while the same procedure taking `VAR v` accepts all of them.

### Forward references and recursive types

The pending-base mechanism handles every shape the plan claims. A self-referential inline pointer field (`Node = RECORD n: INTEGER; next: POINTER TO Node END`), two mutually recursive records connected only by pointers, two pointer constructors waiting on one later record, a pointer nested inside an inline record base whose own base is declared later, and a procedure-local forward pair all compile, allocate, and print correctly.

The scope rules hold in both directions. `Pointers.LocalTypes` proves an already visible outer `Node` wins over a same-named record declared later in the procedure, and `PointerForwardBad.Mod` proves a module-level pointer cannot reach a record declared only inside a procedure. A qualified base never becomes pending. `TYPE SelfP = POINTER TO SelfP` resolves, finds a pointer, and reports `pointer base must be a record type, found SelfP` without recursing.

Recovery from an invalid base does not panic. A module declaring `POINTER TO Missing` and `POINTER TO INTEGER` and then using both — `NEW`, field selection, explicit dereference, `LEN` through the bad pointer, passing it as a value parameter, returning it, comparing it against `NIL` and against the other bad pointer — reported the two base diagnostics plus one genuine type mismatch and nothing else. Dependent lowering is skipped silently, which is what the plan asked for.

The shallow `Debug` implementations for `RecordType` and `PointerType` do cut the cycle, but nothing prints them: running the compiler under `RUST_LOG=debug` over `LinkedList.Mod` emits 1364 lines and not one mentions a semantic type. They are insurance, not exercised code.

### Allocation and the collector

The scanned-versus-atomic choice was checked against the IL for every shape in the slice. `PointerGc.Mod` allocates its pointer-free record with `oberon_alloc_atomic(l 4)`, its `RECORD unused: ARRAY 0 OF Link END` with `oberon_alloc_atomic(l 0)`, its `Node` with `oberon_alloc(l 16)`, and its `Holder` — a pointer, an array of pointers, and a nested record of pointers — with `oberon_alloc(l 40)`. Each size is the exact payload with no header. `Pointers.Mod` allocates its empty record with `oberon_alloc_atomic(l 0)` and the program confirms two simultaneously live empty allocations are both non-`NIL` and unequal.

Reading `Type::contains_pointers` against the layout rules found no shape where a pointer occupies storage and the atomic allocator is chosen: a pointer answers true directly, an array answers only when its length is positive, a nested record contributes its precomputed answer, and a pointer field whose own base is still pending answers true without consulting that base.

`PointerGc` prints `7100050050011121` after the change described under the test findings, which is a count of 1000 and a value sum of 500500 for a list retained across 42 collections. Generated code names only `oberon_alloc` and `oberon_alloc_atomic`; BDWGC entry points appear nowhere outside `runtime/oberon.c`.

`NEW` evaluates its target's selectors once, before allocating. `build/Pointers.ssa` shows `NEW(holder.slots[Index()])` calling `Index`, checking the index, forming the element address, and only then calling the allocator, and `indexCalls` prints `1`.

### `NIL`

`NIL` works in every position the plan lists. Probes confirmed a value actual, a `RETURN NIL` from a pointer function, and equality against a pointer-valued function result. `CONST None* = NIL` is exported and used by a client. `NIL = NIL` folds in a constant declaration.

Misuse is rejected everywhere without a panic. `ORD`, unary minus, `~`, `IN`, `INC`, `ABS`, assignment to a non-pointer, `CASE`, a condition, and arithmetic each produce their own message for a pointer or for `NIL`, and `ORD(NIL)` reports `argument 1 has type NIL, expected CHAR or BOOLEAN or SET`.

`Type::Nil` had doubled as the operand-type marker for a pointer comparison in `Analyzer::lower_relation`: when the two sides were compatible pointers rather than identical ones, the branch yielded `Type::Nil` and the emitter read `ir::Ty::Pointer` off it. The instruction was right and the value never escaped that one call, so this was a readability problem rather than a defect — the operand type of `p = q` was internally NIL. That branch now takes whichever side is a pointer, so only a comparison of two `NIL`s keeps the pseudo-type, and both answer with the same class. No emitted instruction changed.

### Compatibility and identity

Ordinary pointer value compatibility and the stricter `VAR` rule are both in place and both tested. `Change(p2)`, passing a separately constructed pointer with the same base to a `VAR P` formal, is rejected; `Take(p2)` and `p := p2` are accepted. Where the two types print alike the identity hint fires: a procedure-local `P` passed to a module-level `VAR v: P` formal reports `argument 1 has type P, expected P: these are different pointer types, and each POINTER in the source declares its own`.

Across modules, `tests/corpus/modules/pointer-api/` proves descriptors are shared rather than rebuilt. A client's own `POINTER TO PointerTypes.Record` is value-compatible with the exported `PointerTypes.Pointer`, the exported procedures accept and return it, and `NEW` through an exported pointer whose base record is private uses that hidden record's size and its scanned allocation kind. `tests/errors/modules/pointer-private/` confirms an unmarked field stays invisible through an imported pointer, with a message that does not leak the field's name.

The grammar boundaries are right. An inline `POINTER TO` is accepted in a variable declaration and a record field, where Report 6 admits a full `type`, and rejected in a formal parameter and a result position, where the Report admits only a qualident.

### Regressions

Pointer-free code generation is byte-identical, both as the slice was delivered and after this review's fixes. Compiling every module in `tests/corpus` from commit `0a54908` and from the working tree and comparing the emitted IL gave 66 comparisons and one difference, `PredefinedShadow.ssa`, whose source this slice extended to shadow `NEW`. `build/Params.ssa` still hashes to `bc6627e4199e252930401829769ea56477ae76987738f442f8111d8121233291`, the value the two earlier reviews recorded.

Executable `LEN` still does its work. `n := LEN(a[Index()])` and `n := LEN(p.row[Index()])` in statement position both call `Index`, bounds-check its result, nil-check the pointer where there is one, and then answer with the static length; the counter reaches two. So the defect above is confined to the required-constant path, and Slice 10's rule for the executable path is intact.

The deferred features stop where they did. `IS` reports `not yet supported: IS relations`, record extension and `PROCEDURE` types and open arrays remain parse errors, and a type guard `p(R)` reports `'p' is not a procedure` — the pre-slice state, reached without a panic.

## Declined changes

The review does not ask for pointer equality to be widened, for `NIL` to be comparable with an integer, or for the `VAR` rule to be relaxed to matching bases. Each of those is a decision the plan reached against the Report and the vendored compilers, and the corpus pins the result.

The review does not ask for the `Rc` cycle between a record descriptor and a pointer descriptor to be replaced with a type arena. The cycle is bounded by the source type graph and lives only for the process, which the plan already weighed and accepted.

The review does not ask for a nil check to be elided where data flow could prove the pointer is non-null. There is no optimization pass, and the plan's reason for emitting the check unconditionally — that the literal invariant is easier to inspect — is the right one at this stage.

The review does not ask for `GC_set_all_interior_pointers(1)` to be removed on the grounds that the installed collector already defaults to it. The default is a property of how libgc was built, not of the language, and the compiler depends on the behaviour.

## What this review changed

`src/sema.rs` gained `Analyzer::const_index_call` and its two call sites in the required-constant `LEN` walks, and `Analyzer::lower_relation` now names a pointer operand type rather than the `NIL` pseudo-type. `src/driver.rs` sorts diagnostics by position before printing. `runtime/oberon.c` gained comments on the interior-pointer setting and the nil check.

`tests/corpus/Pointers.Mod` gained the pointer alias, whole-record assignment through a pointer, executable `LEN` through both selector forms, the folded `NIL` constant, and the allocation loop before the pointer array is read back. `tests/corpus/PointerGc.Mod` retains its list through heap fields alone across the churn and now checks the node values as well as the count. `tests/errors/PointerLenBad.Mod` gained the three call-in-index cases. `tests/errors/PointerForwardBad.expected` follows the new diagnostic order.

[docs/dev/architecture.md](../architecture.md) records why the required-constant `LEN` path skips nothing observable. [docs/dev/code-style.md](../code-style.md) records the diagnostic ordering guarantee.

## Verification

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `git diff --check` all pass after every change above: 36 unit tests and two integration tests, the latter compiling and running every module in `tests/corpus`, `tests/errors`, and `tests/failures`.

The probe programs written for this review were deleted afterwards, and the temporary worktrees at `0a54908` used for the two regression comparisons were removed. Each probe compiled and ran, and the outputs quoted above are what it printed. The two corpus expectations that changed were hand-derived first and then compared against what the programs printed, rather than recorded from the run.
