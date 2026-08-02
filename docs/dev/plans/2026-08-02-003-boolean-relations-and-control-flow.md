# Slice: BOOLEAN, relations, and control flow

## Context

The compiler currently handles straight-line INTEGER arithmetic: one type, one basic
block per function, assignment and procedure-call statements only. This slice adds the
BOOLEAN type, the relational operators, the logical operators `&` / `OR` / `~`, and the
structured statements `IF`, `WHILE` (with Oberon-07's ELSIF branches), and `REPEAT`.

Why now: every future slice (procedures, arrays, records) needs branching to write a
corpus test that proves anything; a second type forces `Symbol` to carry type
information while that change is still small; and control flow forces basic blocks into
codegen before procedures pile calling conventions on top.

Out of scope, deliberately: `FOR`, `CASE`, function calls in expressions, the typed IR
(deferred to the procedures slice), and `IN` / `IS`.

The lexer already produces every token this slice needs (`True`, `False`, `Tilde`,
`Amp`, `Or`, `Eq`..`Ge`, `If`..`Until`). **No lexer changes.**

## Design decisions

- **Short-circuit is mandatory, not cosmetic.** Report 8.2.2: `p & q` means
  "IF p THEN q ELSE FALSE". Expressions can't call functions yet, but
  `(y # 0) & (x DIV y = 0)` is the standard guard idiom and eager evaluation would
  divide by zero at runtime. So evaluating an expression may now emit branches.
- **QBE does not require SSA input** — temps may be assigned in multiple blocks and QBE
  converts to SSA itself. Short-circuit lowering therefore assigns one result temp in
  both arms and joins, no phi.
- **BOOLEAN is a `w` holding 0 or 1.** Both types are 4 bytes (`alloc4 4`,
  `loadw`/`storew`), so codegen needs no per-expression type queries this slice; all
  type checking stays in sema.
- **Equality (`=`, `#`) applies to both types; ordering (`<` `<=` `>` `>=`) to INTEGER
  only** (Report 8.2.4, restricted to the types we have).
- **Sema poisons instead of cascading:** expression checking returns `Option<Type>`,
  `None` meaning "already diagnosed" — checks that depend on a failed operand stay
  silent rather than piling invented errors on one mistake.

## Changes by file

### src/ast.rs

- `Expr::Bool { value: bool, pos }`.
- `UnOp::Not`.
- `BinOp`: add `Eq, Ne, Lt, Le, Gt, Ge, And, Or`.
- New `Stmt` variants:
  ```rust
  If     { cond: Expr, then: Vec<Stmt>, elsifs: Vec<(Expr, Vec<Stmt>)>, els: Option<Vec<Stmt>> }
  While  { cond: Expr, body: Vec<Stmt>, elsifs: Vec<(Expr, Vec<Stmt>)> }  // Report 9.6: WHILE has ELSIF
  Repeat { body: Vec<Stmt>, cond: Expr }
  ```

### src/parser.rs

- `expression()`: parse one optional relation (`= # < <= > >=`) after
  `simple_expression`, per the grammar (relations don't chain). `IN`/`IS` stay
  `unsupported`.
- `simple_expression()`: handle `Tok::Or` in the add-operator loop.
- `term()`: handle `Tok::Amp` in the mul-operator loop.
- `factor()`: `Tok::True`/`Tok::False` → `Expr::Bool`; `Tok::Tilde` → `UnOp::Not`
  applied to a recursive `factor()` (grammar: `factor = ... | "~" factor`).
- `stmt_seq()`: route `Tok::If`/`Tok::While`/`Tok::Repeat` to new methods
  (`FOR`/`CASE` remain `unsupported`). The existing loop/termination logic already
  handles bodies ending at `ELSIF`/`ELSE`/`END`/`UNTIL`.
- New: `if_statement()` (IF/THEN/{ELSIF}/[ELSE]/END), `while_statement()`
  (WHILE/DO/{ELSIF DO}/END), `repeat_statement()` (REPEAT/UNTIL).

### src/sema.rs

- New `Type` enum: `{ Integer, Boolean }` (Copy, PartialEq), with a `Display`-style
  name for diagnostics ("INTEGER"/"BOOLEAN").
- New `Value` enum for folded constants: `{ Int(i32), Bool(bool) }`.
- `Symbol` gains types: `Const(Value)`, `Var(Type)`, `TypeName(Type)`, and
  `Proc { runtime_name, params: Vec<Type> }` (replaces bare `arity` so `Out.Int(TRUE, 0)`
  is caught).
- Pre-seed scope with both `INTEGER` and `BOOLEAN`.
- Var declarations record their resolved type in `Symbol::Var(t)`.
- `eval_const` returns `Result<Value, Diagnostic>`: folds booleans, relations, `&`,
  `OR`, `~` with the same short-circuit semantics codegen emits; type errors inside
  constant expressions are diagnosed here.
- `check_expr` becomes `fn check_expr(...) -> Option<Type>`:
  - arithmetic ops require INTEGER operands, yield INTEGER
  - `&`/`OR`/`~` require BOOLEAN, yield BOOLEAN
  - `=`/`#` require both operands the same type, yield BOOLEAN
  - `<`/`<=`/`>`/`>=` require INTEGER, yield BOOLEAN
- `check_stmt`:
  - assignment: rhs type must equal the variable's declared type
  - calls: check each argument's type against `params`
  - `If`/`While` conditions (including every ELSIF) and `Repeat`'s UNTIL condition
    must be BOOLEAN
  - recurse into all bodies

### src/qbe.rs

- Add a label counter next to the temp counter; labels are `@.LN` or per-construct
  (`@if3.then`) — dotted names can't collide with anything, same trick as `%.t`.
- Straight-line invariant is gone: after emitting a terminator (`jnz`/`jmp`), the next
  line must be a label. QBE allows fallthrough into a labeled block, but emit explicit
  `jmp` everywhere for readability.
- `expr()`:
  - `Expr::Bool` → `"1"` / `"0"`; `Symbol::Const(Value)` → literal 1/0 or the integer
  - relations → `ceqw` / `cnew` / `csltw` / `cslew` / `csgtw` / `csgew` (both operand
    types are `w`, signed compares)
  - `~x` → `ceqw x, 0`
  - `a & b` / `a OR b` → short-circuit blocks: evaluate `a`, `jnz` past `b`'s
    evaluation, assign one result temp in both arms (non-SSA is fine), join.
- `stmt()` gains three lowering cases:
  - `If`: chain of test blocks, each `jnz` to its body or the next test; every body
    `jmp`s to a shared end label
  - `While`: top-of-loop test label; each guard's body jumps *back to the test label*
    (Report 9.6: ELSIF guards are re-checked from the top each iteration); all guards
    false → end label
  - `Repeat`: body label, evaluate cond, `jnz cond, @end, @body`

### Unchanged

`src/lexer.rs`, `src/diag.rs`, `src/driver.rs`, `src/main.rs`, `runtime/oberon.c`,
`tests/corpus.rs` (the harness picks up new modules automatically).

## New corpus modules

`tests/corpus/` (compile, run, compare stdout):
- `If.Mod` — IF/ELSIF/ELSE chains, nested IF
- `While.Mod` — a loop with a real trip count; include a WHILE-with-ELSIF (e.g. the
  Report's gcd-by-subtraction example)
- `Repeat.Mod` — REPEAT/UNTIL, body-executes-at-least-once case
- `ShortCircuit.Mod` — `(y # 0) & (x DIV y = 0)` with `y = 0`: proves `&` guards the
  division; same for OR
- `BoolOps.Mod` — `~`, `=`/`#` on booleans, boolean VARs and CONSTs, boolean assignment

`tests/errors/` (must fail with exact stderr):
- `CondNotBool.Mod` — `IF x THEN` with INTEGER x
- `AssignMismatch.Mod` — `b := 1` for BOOLEAN b
- `OrderOnBool.Mod` — `TRUE < FALSE`
- `AndOnInt.Mod` — `1 & 2`
- `BadArgType.Mod` — `Out.Int(TRUE, 0)`

## Verification

1. `cargo test` — full corpus (existing 5 + new modules) plus error corpus must pass.
2. Spot-check generated IL: `cargo run -- tests/corpus/ShortCircuit.Mod` then read
   `build/ShortCircuit.ssa` — confirm `&` emits a `jnz` before the divide, and `qbe`
   accepts multi-assigned temps.
3. `./build/While` etc. by hand once, to see real output.

## Order of work

1. ast.rs + parser.rs (new nodes, new statements) — `cargo build` green with sema
   temporarily rejecting the new nodes via `todo!()` is fine mid-step
2. sema.rs (Type/Value/Symbol rework, then checking rules)
3. qbe.rs (labels, relations, short-circuit, statement lowering)
4. corpus modules, then `cargo test`
