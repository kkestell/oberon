# Review: Nested procedures and lexical scope

Reviewed against the uncommitted tree on top of commit `f897003` ("update
roadmap"). The implementation follows
[the Slice 5 plan](../plans/2026-08-02-005-nested-procedures-and-lexical-scope.md).

## Verdict

The slice is complete against its plan. Nested procedures compile at arbitrary
depth. Their symbols contain the full declaration path. They can use their own
objects, module objects, and constants or procedures from enclosing scopes.
They cannot use variables or parameters from an enclosing procedure.

No defect was found in the compiled behavior. Four non-blocking observations
came out of the review, and all four were applied before the slice was
committed. One added a comment, one reworded a diagnostic, and two widened
what the corpus pins. This document describes the tree after those changes.

## Verified behavior

`Nested` declares `R` and `F` inside `Q`, which is inside `P`. `R` updates the
module variable `total`. It uses constants declared by both `P` and `Q`. It
calls the module procedure `AddGlobal`. `Q` passes its local `q` to `R` as a
`VAR` argument. `P` adds its own local `p` into `total` both before and after
it calls `Q`, so every procedure in the chain touches the same module
variable, and the parent's local storage has to survive the call to see the
right answer. The program prints:

```text
26 42
```

`NestedRec` exercises three recursion and visibility shapes. `Count` calls
itself. The child `R` calls its parent `Q`, while `Q` calls `R`. The later
sibling `Later` calls the earlier sibling `Earlier`. The program prints:

```text
4 23  7
```

`Shadow` gives the name `x` to a module variable, an outer local, an inner
local, and a parameter. Each use resolves to the innermost declaration. The
outer procedure also contains a procedure with the same name as its parent.
The program prints:

```text
2  3  4  5  1
```

`NotAccessible` reports five errors. They cover reading an enclosing local,
assigning to it, passing it as a `VAR` actual, reading an enclosing value
parameter from two levels down, and reading an enclosing `VAR` parameter.
Every error uses the dedicated `not accessible` diagnostic.

`SiblingRef` reports `undeclared identifier 'Second'` when an earlier nested
procedure calls a later sibling. `NestedDup` reports two duplicate declaration
errors. One is for a local that duplicates a parameter. The other is for two
nested siblings with the same name.

## Generated IL

`build/Nested.ssa` contains ordinary QBE functions named
`$Nested.P.Q.R`, `$Nested.P.Q.F`, `$Nested.P.Q`, and `$Nested.P`. `R` has only
its declared value parameter and `VAR` parameter. `F` has only its declared
value parameter. No function has a static link or another hidden parameter.

The existing `Params` module produced byte-identical IL before and after the
slice. Both copies have SHA-256
`bc6627e4199e252930401829769ea56477ae76987738f442f8111d8121233291`.

## Code review

`ProcDecl` now owns its nested declarations. The parser fills that list by
calling `proc_declaration` recursively at the grammar's declaration point.
The old unsupported diagnostic and its negative test are gone.

Semantic analysis declares each procedure heading before opening its local
scope. This preserves direct recursion. It also makes a parent procedure
visible inside a child. The analyzer saves the parent's `ProcBuilder` while it
lowers a child. It restores that builder before lowering the parent's body.
Each generated procedure therefore retains its own slots, temporaries, and
labels.

Nested declarations are analyzed before their parent's statements. This
matches source order. It also keeps diagnostic order stable. Earlier sibling
procedures are visible, while later siblings are not yet declared.

Name lookup records the scope index of the nearest declaration. A variable
from a scope between the module and current procedure is rejected. Lookup does
not fall through that inaccessible declaration to a module object with the
same name. Constants and procedure symbols remain usable across intermediate
procedure scopes, as the plan requires.

The IR, QBE backend, runtime, and corpus harness are unchanged.

## Findings

None of these changed what the compiler accepts or rejects. All four were
applied on the spot rather than deferred, because each was a few lines.

### The scope-stack invariant was not written down in the code

`resolve` decides that a declaration sits in an enclosing procedure by
comparing its scope index against two numbers. Index zero is the module scope
and is allowed. The last index is the procedure being analyzed and is allowed.
Anything between the two is rejected when it is a variable.

That arithmetic is only correct because a procedure body is the one and only
thing that pushes a scope. Nothing in sema.rs said so. The plan says it, and
the plan also says that a future slice which pushes a scope for anything else
breaks the rule silently. `resolve` would then start accepting an enclosing
procedure's variables again, and no existing test would notice, because the
error corpus only exercises procedure nesting.

The check now carries a comment naming both the Report rule it enforces and
the stack shape it assumes, and saying what a future slice has to do instead
if it ever pushes a scope for something other than a procedure body.

### The diagnostic said "variables" when the object was a parameter

The old message read:

```text
'param' is not accessible: a nested procedure cannot use the variables of an enclosing procedure
```

`param` is a formal parameter, and Report 10.1 calls it that. The message now
says "the variables or parameters of an enclosing procedure", which is the
rule as the Report states it.

### `Nested` had a procedure in the chain that touched no module variable

The roadmap's gate for this slice asks that each procedure in the deep chain
access the same module variable. The outer procedure `P` used to assign `99`
to its own local `p` and then call `Q`, reading and writing nothing at module
level.

The local `p` was also never printed. Its only job was to make the parent's
storage exist across the analysis of the nested procedures. If the analyzer
lost the parent's builder, `p := 99` would panic rather than compile, so the
store did prove something. It did not prove the stored value survives, because
no output depended on it.

`P` now sets `p` to `5`, adds it into `total`, calls `Q`, and adds it into
`total` again. The second addition can only give the right answer if the
parent's slot came through the nested analysis and the nested call intact.
The module's output moved from `16 32` to `26 42`, which is what the source
says it should be.

### Nothing pinned an enclosing `VAR` parameter as inaccessible

`NotAccessible` covered three uses of an enclosing local and one use of an
enclosing value parameter. A `VAR` parameter was missing. It is a different
symbol shape: its address is a temporary holding a pointer, while a value
parameter and a local both get a stack slot.

The accessibility check never looks at the address, so the omission was low
risk, and the behavior was already right. `NotAccessible.Mod` now declares
`ByRef(VAR ref: INTEGER)` with a nested `UseRef` that reads `ref`, and the
fifth pinned diagnostic is:

```text
tests/errors/NotAccessible.Mod:38:13: 'ref' is not accessible: a nested procedure cannot use the variables or parameters of an enclosing procedure
```

## Verification

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and
`cargo test` all pass. The three gate binaries were also run directly. Each
exited successfully and printed the output recorded above. A search of
parser.rs found no remaining `nested procedures` unsupported diagnostic.

The byte-identical claim was checked by building the compiler at commit
`f897003` in a scratch worktree and compiling every module both compilers
share. All sixteen `.ssa` files have the same SHA-256 under both compilers.
That comparison was run again after the four fixes landed, with the same
result. Only the diagnostic string changed in sema.rs, so no generated code
moved.

`resolve` is the only place that reads a scope, and `declare` is the only
place that writes one. `procedure` is the only place that pushes or pops.
That was confirmed by searching sema.rs for `scopes`, and it is what makes the
single accessibility check cover reads, assignment targets, `VAR` actuals, and
type positions alike.

### Probes run against the tree

These modules were compiled and, where they compile, run. They are not part of
the corpus; they were written to attack the slice from angles the gate does
not cover.

An enclosing `VAR` parameter is rejected. That probe became the `ByRef` case
now in `NotAccessible.Mod`. A constant expression that names an enclosing
variable gets the same accessibility error rather than a confusing one about
constant folding.

A four-level chain works. The innermost procedure calls a function declared
beside it, calls an uncle procedure, and recurses into its great-grandparent.
It also uses a constant declared three levels up. The program prints `113`,
which is what the source says it should.

A nested procedure named `Helper` inside a procedure that also sees a
module-level `Helper` resolves to the nested one. The module body still
reaches the module-level one. The program prints `21`.

A nested procedure whose parent local shadows a module variable of the same
name reports the accessibility error. Lookup does not fall through to the
module variable.

Declaration point holds across two levels: a procedure nested two deep cannot
see a later sibling of its grandparent, and gets `undeclared identifier`.

Diagnostic order follows source order. A module with one error in the parent's
local declarations, one inside a nested procedure, and one in the parent's
body reports them in that order. This is what the plan's placement of the
nested-procedure loop was for.

The grammar's declaration point is enforced. A `PROCEDURE` after the parent's
`BEGIN`, and a `VAR` section after a nested procedure, are both parse errors.

Three hundred levels of nesting compile, assemble, link, and run. The deepest
symbol is roughly seventeen hundred characters long, and neither QBE nor the
system assembler objected.
