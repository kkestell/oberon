# Slice: Nested procedures and lexical scope

## Context

After Slice 4 the compiler supports module-level procedures over INTEGER and
BOOLEAN, with value and `VAR` parameters, recursion, and a typed IR between
sema and QBE. The parser still rejects a `PROCEDURE` token inside a procedure
body with "not yet supported: nested procedures". This slice removes that
restriction and completes procedure nesting for the scalar language.

Report sections implemented: §4 (scope extends from the declaration point to
the end of the block; no identifier may denote two objects in one scope), §10
(procedure declarations may be nested; the visibility rule for procedure
bodies; recursive activation), and §10.1 (formal parameters are local to the
procedure). With this slice the `DeclarationSequence` production is complete
except for `TYPE` declarations.

### Oberon-07 removed access to enclosing procedures' variables

Classic Oberon let a nested procedure read and write the locals and
parameters of its enclosing procedures, which forces compilers to carry a
static link or equivalent capture mechanism. Oberon-07 removed the feature.
Report §10 states the visibility rule for a procedure body in one sentence:

> In addition to its formal parameters and locally declared objects, the
> objects declared globally are also visible in the procedure.

Formal parameters, local objects, global objects — and nothing else. The
locals and parameters of an *enclosing* procedure are deliberately absent
from that list. §4's scope rule makes the enclosing procedure's names known
textually inside a nested procedure, but §10 withholds access to their
objects.

All three vendored compilers enforce this:

- Project Oberon, `ORG.MakeItem`: an object with `y.lev > 0` and
  `y.lev # curlev` is marked "not accessible" unless its class is `Const`.
  In Wirth's symbol table, class `Const` covers both constants and
  procedures, so intermediate-level constants and procedure calls pass and
  intermediate-level variables and parameters are rejected.
- oberonc, `OJB.thisObj`: an object found in a scope that is neither the
  innermost nor the global one is marked "not accessible" when its class is
  in `{Var, Par, ParStruct}` — the same rule, stated from the other side.
- OBNC, `Table_At`: the lookup consults only the current scope and the global
  scope, so anything declared at an intermediate level — even a constant —
  fails as an undeclared identifier.

So this slice implements the Report's rule. A nested procedure may use its
own declarations, module-level declarations, and the constants and
procedures of enclosing procedures. It may not use the variables or
parameters of enclosing procedures, and that restriction gets a dedicated
diagnostic.

Declined: classic-Oberon captures via a hidden static link. The machinery
(frame records grouping captured slots, a pointer type in the IR, link
chaining at every call site) would exist to accept programs that conforming
Oberon-07 compilers reject. That is a language extension, an explicit
non-goal.

### What remains unsupported after this slice

- `FOR`, `CASE`, and the predefined procedures (`ABS`, `ODD`, `INC`, `DEC`,
  `ASSERT`, and the rest) — Slice 6.
- `TYPE` declarations, export marks, and imports other than `Out` — Slices
  6–7.
- Every type beyond INTEGER and BOOLEAN, and with them `ARRAY OF` formal
  types and structured parameters — Slices 8+.
- Procedure types and procedures as values — Slice 16. The Report already
  guarantees (§6.5) that a procedure assigned to a procedure variable "must
  not be declared local to another procedure", so nothing built in this
  slice needs to escape its enclosing activation, ever. That is why no
  static link is needed even later.

The floored `DIV`/`MOD` overflow near the INTEGER limits, found in the Slice
4 review, remains an open follow-up. It is independent of nesting and is not
folded into this slice.

## Design decisions

### Visibility is enforced in `resolve`, keyed on scope position and symbol kind

`resolve` already walks the scope stack from innermost to outermost. It
learns one new rule: if the name is found in a scope that is neither the
innermost scope nor the module scope, and the symbol is a `Symbol::Var`,
resolution fails with:

```
'x' is not accessible: a nested procedure cannot use the variables of an enclosing procedure
```

`Symbol::Var` covers module variables, locals, value parameters, and `VAR`
parameters; module variables live in the module scope and are exempt, leaving
exactly the classes the reference compilers reject. Constants, procedures, and
type names pass through unchanged, wherever they were declared. Because every
consumer of names (`lower_expr`, `addr_of`, `var_actual`, `resolve_type`,
`eval_const`, the const checkers) goes through `resolve`, one change covers
reads, assignment targets, and `VAR` actuals alike.

Testing scope *position* rather than a recorded nesting level is only valid
because a procedure body is the sole thing that pushes a scope, so the stack
is exactly `[module, outermost proc, ..., current proc]`. Oberon has no block
scopes, and the `FOR` and `CASE` statements arriving in Slice 6 introduce
none, so the invariant holds. A future slice that pushes a scope for anything
other than a procedure body breaks this rule and must switch to storing a
level on the symbol.

The diagnostic deliberately explains itself. Anyone arriving from Pascal or
classic Oberon will expect the access to work; "undeclared identifier" would
be misleading (the identifier is declared and in scope), and Wirth's bare
"not accessible" assumes the reader already knows the rule.

Declined: OBNC's stricter behavior, where intermediate constants are also
invisible. The Report's §10 sentence read literally does exclude constants,
but Wirth's own compiler exempts them, oberonc follows, and a constant has
no storage — there is no activation-record reason to reject it. Two of the
three references, including the authority, allow it; we follow them. The
positive corpus pins the choice.

### Intermediate procedures stay callable, which permits parent recursion

A nested procedure R declared inside Q, itself inside P, may call: itself,
its earlier siblings, Q's earlier siblings, Q itself, and any module-level
procedure. All of these are `Symbol::Proc` found somewhere on the scope
stack, so no new call machinery is needed — the visibility rule above simply
does not fire for procedures. Mutual recursion between Q and R (parent calls
child, child calls parent) becomes expressible because Q's heading is
declared in P's scope before Q's body — including R — is analyzed. The gate
exercises it.

A later sibling remains invisible for the same declaration-point reason as
at module level: siblings are analyzed in source order, heading first, so a
reference to a later sibling is an ordinary "undeclared identifier".

### Nested procedures are analyzed where they appear in the source

Wirth's `ORP.ProcedureDecl` orders the work as: enter the heading in the
enclosing scope, open a scope, formal parameters, `Declarations`, then the
nested procedure loop, then the body. This slice's sema follows the same
order, and the placement of the nested loop is load-bearing beyond mere
symmetry.

Diagnostics are printed in the order sema collects them — `report` in
driver.rs does not sort — and the error corpus compares stderr byte for byte.
Analyzing nested procedures between the parent's local declarations and the
parent's body is what keeps collection order equal to source order, because
that is where they sit in the source. Analyzing them after the parent's body
would compile identical code while scrambling every multi-error `.expected`
file in a way that looks like a test bug rather than an ordering choice.

### Symbols mangle as the full nesting path

A nested procedure's symbol is its enclosing procedure's symbol plus a dot
and its own name: R inside Q inside P in module M becomes `$M.P.Q.R`. Oberon
identifiers cannot contain dots, so distinct paths give distinct symbols, no
path can collide with a C runtime symbol, and the Slice 4 verification that
qbe and cc accept dotted symbols already covers these.

Two procedures share a path only if they share both a name and an enclosing
chain, which means they were declared in one scope. That is a duplicate
declaration, so it is diagnosed, and any diagnostic discards the whole
program before emission. Colliding symbols are therefore unreachable in
emitted IL — the guarantee comes from the discard rule, not from the
duplicate check alone.

### No IR, backend, runtime, or harness change

Because no nested procedure can touch an enclosing frame, a nested procedure
is compiled exactly like a module-level one: its own params, its own slots,
its own temps and labels (`ProcBuilder` is already per-procedure). `ir.rs`
and `qbe.rs` are untouched, and the IL for every existing module must come
out byte-identical. The only structural change in sema is that `procedure`
becomes recursive: it stashes the current `ProcBuilder`, analyzes each nested
declaration (which installs and finishes its own builder), and restores the
outer builder before lowering the outer body. Nested procedures therefore
land in `Program::procs` before their parents; QBE does not care about
definition order.

No new dynamic checks are introduced, so there are no new failure-corpus
modules and no runtime edits.

## Changes by file

### src/ast.rs

`ProcDecl` gains `procs: Vec<ProcDecl>`.

### src/parser.rs

In `proc_declaration`, replace `unsupported("nested procedures")` with the
same loop `module` uses: `while *self.peek() == Tok::Procedure`, parse a
nested `proc_declaration`. The grammar places nested procedures after the
CONST and VAR sections and before `BEGIN`, which is where the loop sits.

### src/sema.rs

- `procedure` takes the enclosing symbol prefix (the module name at the top
  level, the enclosing procedure's mangled symbol below) and builds
  `symbol = format!("{prefix}.{name}")`.
- `procedure` stashes `self.current` on entry and restores it before
  returning, so nested analysis can run between local declarations and body
  lowering. Between `local_declarations` and `lower_stmts`, it loops over
  `declaration.procs` recursively.
- `resolve` reports which scope the name was found in; a `Symbol::Var` from
  an intermediate scope becomes the new "not accessible" diagnostic.

Roughly thirty lines. Nothing else in sema knows how deep it is.

### Tests

`tests/errors/NestedProc.Mod` and its `.expected` are deleted; the parser
diagnostic they pin no longer exists. New modules are listed next.

## New corpus modules

`tests/corpus/` (compile, run, compare stdout):

- `Nested.Mod` — the workhorse. P contains Q contains R. R reads and writes
  a module variable from two levels down, uses a CONST declared in P and one
  declared in Q, and calls a module-level helper. Q calls R with value and
  `VAR` arguments; P calls Q; the module body calls P. Includes a nested
  function procedure whose RETURN is computed from its own locals and a
  global.
- `NestedRec.Mod` — recursion shapes: a nested procedure that recurses
  directly; mutual recursion between parent Q and child R; a later nested
  sibling calling an earlier one. Results accumulate in module variables and
  print at the end.
- `Shadow.Mod` — a module variable, a P local, and a Q-in-P local all named
  `x`, each written and printed where it is innermost, with the module `x`
  printed last to prove the inner writes never touched it; a nested
  procedure whose parameter shadows a module variable; and a nested
  procedure sharing its parent's name. That last case needs its assertion
  chosen deliberately: inside the child, the child's own declaration is the
  innermost one, so a self-named call recurses into the child and the parent
  becomes unreachable by name. The module must observe which one ran rather
  than assuming the parent was called.

`tests/errors/` (must fail with exact stderr):

- `NotAccessible.Mod` — one nested procedure reads the parent's local, one
  assigns to it, one passes it as a `VAR` actual, and one reads the parent's
  value parameter from two levels down. Four pinned "not accessible"
  diagnostics, covering every consumer path through `resolve`.
- `SiblingRef.Mod` — a nested procedure calls its later sibling; pins
  "undeclared identifier" at the call.
- `NestedDup.Mod` — inside a nested procedure, a local duplicates a
  parameter name; and two nested siblings share a name. Pins one
  "is already declared" diagnostic for each, in source order.

## Verification

1. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
   `cargo test`.
2. IL inspection, since the slice touches calling structure: in
   `build/Nested.ssa`, every nested procedure must appear as an ordinary
   `function $Nested.P.Q...` with no extra parameters — the greppable proof
   that no hidden mechanism exists. An unchanged module from the existing
   corpus must produce byte-identical IL before and after the slice.
3. Run the gate binaries by hand (`./build/Nested`, `./build/NestedRec`,
   `./build/Shadow`) and read the real output.
4. Grep parser.rs for `nested procedures` — the unsupported diagnostic must
   be gone.

## Order of work

1. Delete the `NestedProc` error test.
2. ast.rs and parser.rs — recursive procedure declarations.
3. sema.rs — prefix-based mangling, builder stash, recursive analysis.
4. sema.rs — the accessibility rule in `resolve` and its diagnostic.
5. Gate modules and `.expected` files; regression modules for anything found
   on the way.
6. The verification list, including the byte-identical-IL check.
