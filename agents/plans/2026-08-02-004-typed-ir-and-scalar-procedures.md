# Slice: Typed IR and scalar procedures

## Context

The compiler handles a scalar INTEGER and BOOLEAN language: constants, module
variables, assignment, `IF`/`WHILE`/`REPEAT`, short-circuit logic, and calls to
the built-in `Out`. It has one flat scope and lowers the AST directly to QBE
text. Two stages currently share semantic work badly: sema resolves and
type-checks every designator, and then qbe.rs resolves each one a second time
against the same scope, panicking if the two walks disagree. qbe.rs also owns
semantic knowledge it should not have, such as the floored `MOD` formula and
short-circuit lowering.

Procedures are the feature that breaks this arrangement. A `VAR` parameter is
an address while a value parameter is a value. A module variable lives in
static data while a local lives in a stack frame. Calls need argument passing
and results need returns. The typed IR described in the project architecture
exists to make exactly these distinctions explicit, so this slice introduces
the IR and module-level procedures together, as the roadmap requires.

Report sections implemented: §4 (scope of local declarations), §8.1 (function
designators in expressions), §9.2 (procedure call statements), §10 (procedure
declarations and the RETURN clause), and §10.1 (value and variable parameters),
all restricted to INTEGER and BOOLEAN. The slice also adds the runtime check
for `DIV` and `MOD` by zero and the test harness's third corpus class for
expected runtime failures.

Valid Oberon that remains unsupported after this slice:

- Nested procedure declarations (Slice 5). The parser diagnoses them as
  unsupported rather than panicking.
- `FOR`, `CASE`, and the predefined procedures such as `ABS`, `ODD`, `INC`,
  `DEC`, and `ASSERT` (Slice 6).
- Export marks, `TYPE` declarations, and imports other than `Out` (Slices 6–7).
- Every type beyond INTEGER and BOOLEAN, and with them `ARRAY OF` formal
  types and structured parameters (Slices 8+).
- Procedure types and procedures as values (Slice 16).

One consequence of the Report worth recording: module-level mutual recursion is
inexpressible in Oberon-07. §4 requires declaration before use, §10 makes a
procedure visible inside its own body for direct recursion, and the language
has no forward declarations. Sema therefore processes procedures in source
order, declaring each heading before checking its body. A call to a
later-declared sibling is an ordinary "undeclared identifier" error, and that
is the correct behavior, not a limitation to lift later.

## Design decisions

### The IR is a flat, typed, QBE-shaped instruction list

`ir.rs` defines data only: a `Program` of globals and procedures, each
procedure holding a flat `Vec<Inst>` in which labels, branches, loads, stores,
calls, and returns are explicit instructions. Sema does all name resolution
and all type decisions while building it. qbe.rs becomes a printer that maps
each instruction to one or two lines of IL and never looks at the AST or the
scope again.

Declined: a structured IR (statement trees with resolved names). It would keep
control-flow lowering in the backend, which is the semantic leakage this slice
exists to remove. Also declined: basic-block structs with separate terminator
fields. They make the "label follows terminator" invariant structural, but
they add builder ceremony for no current payoff; the flat list mirrors both
the current emitter and QBE's own text format. Blocks can be introduced later
if a pass ever needs the CFG.

### Sema checks and lowers in a single pass

`sema::analyze` changes signature to
`fn analyze(&ast::Module) -> Result<ir::Program, Vec<Diagnostic>>`. Expression
lowering returns `Option<(ir::Value, Type)>`, keeping the existing poisoning
scheme: `None` means "already diagnosed, stay quiet". Instructions are emitted
optimistically into the current procedure's buffer; if any diagnostic exists
at the end, the whole program is discarded and `Err` is returned, so the IR
by construction only ever describes a valid module.

Declined: a separate lowering pass after checking. It would have to resolve
every designator and rebuild every scope a second time — precisely the
duplication between sema and qbe.rs that the IR is meant to end.

### Scopes become a stack, symbols carry storage

The single `Scope` map becomes a stack of maps: the module scope plus one
local scope per procedure body (Slice 5 will deepen the stack for nesting).
Lookup walks from innermost outward. `declare` checks only the innermost
scope, so a local may shadow a module name; that is legal per §4. The
first-declaration-wins rule for duplicates stays.

`Symbol::Var` grows from a bare type to `Var { ty: Type, addr: ir::Addr }`.
One variant now covers module variables, locals, value parameters, and `VAR`
parameters — they differ only in the address they carry. This is also what
makes the `VAR`-actual check simple: an actual is acceptable exactly when it
resolves to a `Symbol::Var` of identical type (§10.1).

`Symbol::Proc` becomes `Proc { symbol: String, params: Vec<(bool, Type)>,
ret: Option<Type> }`, where the bool marks a `VAR` parameter and `symbol` is
the final QBE name. The `Out` built-ins are re-expressed in this same shape,
so user procedures and runtime procedures go through one call-checking and
one call-lowering path.

### Storage classes

- Module variables become QBE data objects: `data $Mod.x = align 4 { z 4 }`.
  This is what lets procedures reach them; they leave the initializer's frame
  as the roadmap requires. Zero initialization is what a data object gives us;
  the Report leaves variable values undefined, so no test may rely on the
  zeroing.
- Locals and value parameters become frame slots (`alloc4 4`) in their
  procedure. §10.1 says a basic-type value parameter "represents a local
  variable to which the value of the actual expression is initially assigned",
  so each value parameter gets a slot and an explicit entry-block store of the
  incoming argument, and assigning to it is legal.
- A `VAR` parameter is an incoming pointer (QBE class `l`). Its temp is used
  directly as an address; no slot is allocated for it.

### Symbol mangling, verified against qbe and cc

User-level symbols mangle as `$Mod.name` (dot separator). Oberon identifiers
cannot contain dots, so two user symbols can never collide, and a dotted name
can never collide with a C runtime symbol. The synthesized module initializer
is `$.Mod.init`, with a leading dot; no user name can mangle to a leading dot,
extending the existing convention that compiler-internal names (`%.t0`,
`@.if.end0`) start with `.`. Verified today: qbe and cc accept a program using
`data $Mod.x` and `function $.Mod.init()`, and the linked binary runs and
reads the data object back correctly.

Declined: keeping the current `$Mod_init` (a module named `oberon` would
mangle its initializer to `oberon_init`, which the C runtime already exports).
Declined: naming the initializer bare `$Mod` (a module named `puts` would
collide with libc).

The `$main` stub — call `$oberon_init`, call `$.Mod.init`, return 0 — stays in
qbe.rs as target glue. Representing it in the IR would need an export flag on
procedures and buys nothing until the multi-module slice.

### Calls, RETURN, and the function/proper distinction

Arguments are evaluated left to right. A value actual is lowered to a value
and passed as `w`; a `VAR` actual is lowered to an address and passed as `l`.
A `VAR` actual that is not a writable designator (a constant, a literal, an
expression, a procedure) is diagnosed at the argument position.

The grammar puts `[RETURN expression]` in the procedure body, before `END` —
Oberon-07 has no RETURN statement. Sema enforces §10: a function procedure
must end with a RETURN clause whose expression has the result type; a proper
procedure must not have one. A function procedure lowers RETURN to
`Ret(Some(value))`; a proper procedure gets an implicit `Ret(None)`.

Function procedures may only be activated inside expressions (§10), so a
function call in statement position is an error, as is a proper procedure in
an expression. A bare procedure designator used as a value keeps the existing
"cannot be used as a value" diagnostic, which is also the gate's
proper-procedure-as-value negative test. A parameterless function call
requires the empty parentheses (§10.1); without them it is a bare designator
and falls under the same rule.

### DIV and MOD by zero trap at runtime

Lowering emits the zero check itself, before the floored-division sequence:
compare the divisor with zero, branch to a per-site trap block that calls
`oberon_div_by_zero` and ends in `hlt`. The check is skipped when the divisor
is a nonzero integer literal, purely to keep the inspected IL readable. A
literal zero divisor outside a constant context compiles into an unconditional
trap rather than a compile error — constant contexts already diagnose it at
fold time, and the Report does not require rejecting the runtime case.
Declined: making the runtime case a static error too; it adds a rule the
Report does not state.

The runtime gains:

```c
void oberon_div_by_zero(void) { fputs("DIV or MOD by zero\n", stderr); exit(1); }
```

One function serves both operators because one check guards the shared
divisor. The message and the exit status 1 are pinned by the failure corpus.

### Floored DIV/MOD moves out of the backend

The `((x rem y) + y) rem y` sequence currently lives in qbe.rs with a comment
promising it matches sema's constant folding. Lowering now emits it as
explicit IR instructions (`Rem`, `Add`, `Rem`, then `Sub` and `Div` for the
quotient), so both the folded and the runtime semantics live in sema.rs and
the printer stops knowing any Oberon arithmetic rules. The IR's `Div` and
`Rem` are QBE's truncating operations; the Oberon semantics exist only in the
instruction sequences sema chooses to emit.

### A third corpus class for runtime failures

`tests/failures/X.Mod` must compile, and the binary must exit nonzero with
stderr exactly matching `X.expected`. Later slices reuse the class for bounds,
nil, assertion, and type-guard failures. Stdout is not compared for this
class; the failure contract is the status and stderr.

## The IR

Sketch, to fix vocabulary; details may shift during implementation:

```rust
pub struct Program {
    pub module: String,
    pub globals: Vec<Global>,        // Global { symbol, ty } → data $sym = align 4 { z 4 }
    pub procs: Vec<Proc>,            // includes the synthesized $.Mod.init
}

pub struct Proc {
    pub symbol: String,
    pub params: Vec<Param>,          // Param { temp, pass: Value(Ty) | Ref }
    pub ret: Option<Ty>,
    pub slots: Vec<(String, Ty)>,    // locals + value params; alloc4 4 each at entry
    pub insts: Vec<Inst>,
}

pub enum Ty { Int, Bool }            // both print as QBE 'w' today

pub enum Value { Int(i32), Bool(bool), Temp(usize) }

pub enum Addr {
    Global(String),                  // $Mod.x
    Slot(String),                    // %x from the frame alloc
    Temp(usize),                     // a pointer-valued temp: a VAR parameter
}

pub enum Inst {
    Label(String),
    Load  { dst: usize, ty: Ty, addr: Addr },
    Store { ty: Ty, val: Value, addr: Addr },
    Copy  { dst: usize, src: Value },                       // short-circuit joins
    Un    { dst: usize, op: UnOp, arg: Value },             // Neg, Not
    Bin   { dst: usize, op: BinOp, lhs: Value, rhs: Value },// Add..Rem, Eq..Ge
    Call  { dst: Option<(usize, Ty)>, symbol: String, args: Vec<Arg> },
    Jmp(String),
    Br    { cond: Value, then: String, els: String },
    Ret(Option<Value>),
    Halt,                                                   // hlt, after a trap call
}

pub enum Arg { Val(Ty, Value), Ref(Addr) }                  // w vs l at the call site
```

Invariants, kept by construction in sema and `expect`ed in qbe.rs where cheap:
every instruction after `Jmp`/`Br`/`Ret`/`Halt` is a `Label`; every temp is
assigned before use (QBE itself rejects violations); slot names are unique
within a procedure because the local scope already rejects duplicates.

## Changes by file

### src/ast.rs

- `Module` gains `procs: Vec<ProcDecl>`.
- New declarations:
  ```rust
  pub struct ProcDecl {
      pub name: String,
      pub pos: Pos,
      pub params: Vec<FpSection>,   // FpSection { var: bool, names: Vec<(String, Pos)>, ty: Designator }
      pub ret: Option<Designator>,
      pub consts: Vec<ConstDecl>,
      pub vars: Vec<VarDecl>,
      pub body: Vec<Stmt>,
      pub ret_val: Option<Expr>,    // the [RETURN expression] clause
  }
  ```
- `Expr::Call { callee: Designator, args: Vec<Expr>, pos }` for function
  designators in expressions.

### src/parser.rs

- The declaration loop in `module()` routes `Tok::Procedure` to a new
  `proc_declaration()` instead of `unsupported`. The CONST/VAR section parsing
  is extracted into a helper shared by the module and procedure bodies.
- `proc_declaration()`: heading (`identdef`, so export marks stay diagnosed),
  optional formal parameters, `;`, body declarations, optional `BEGIN`
  statement sequence, optional `RETURN` expression, `END`, and a trailing
  identifier that must repeat the heading name, mirroring the module check.
  A `PROCEDURE` token inside a body is `unsupported("nested procedures")`
  until Slice 5.
- `formal_parameters()`: `FPSection = [VAR] ident {"," ident} ":" FormalType`;
  `ARRAY OF` in a formal type is `unsupported`. The result type is a plain
  designator resolved by sema.
- `factor()`: a designator followed by `(` becomes `Expr::Call` instead of
  `unsupported("function calls in expressions")`.

### src/ir.rs (new)

The types above, `#[derive(Debug)]` throughout, no logic beyond perhaps small
constructors. The IR is dumped with `tracing::debug!` from the driver.

### src/sema.rs

The largest change: sema becomes check-and-lower.

- `analyze` returns `Result<ir::Program, Vec<Diagnostic>>`. Processing order:
  imports, module consts, module vars (declared as globals with mangled
  symbols), then each procedure in source order, then the module body lowered
  into the `$.Mod.init` procedure.
- For each procedure: build the signature by resolving formal types, declare
  the `Symbol::Proc` in the module scope first (direct recursion), then push a
  local scope holding parameters, local consts (folded with the existing
  `eval_const`), and local vars; lower the body; enforce the RETURN rules;
  pop the scope.
- `check_expr` becomes a lowering function returning
  `Option<(ir::Value, Type)>`; a sibling `addr_of` resolves a designator to
  `Option<(ir::Addr, Type)>` for assignment targets and `VAR` actuals.
- Short-circuit `&`/`OR` lowering (branches plus `Copy` into a shared result
  temp) moves here from qbe.rs, as does the floored DIV/MOD sequence and the
  new zero check.
- One call-checking path serves statements and expressions, with the
  function/proper distinction on top: statements require `ret == None`,
  expressions require `ret == Some(t)`.
- Intended new diagnostics, exact wording pinned by the error corpus:
  "argument N must be a variable", "function procedure 'F' must end with
  RETURN", "RETURN expression has type BOOLEAN, expected INTEGER", "proper
  procedure 'P' cannot RETURN a value", "function 'F' cannot be called as a
  statement", plus the existing "'P' cannot be used as a value" now reachable
  through user procedures.

### src/qbe.rs

Rewritten as a printer over `ir::Program`, roughly 150 lines: emit `data`
lines for globals; per procedure emit the signature (`w` value params, `l`
`VAR` params, optional `w` result), `@start`, one `alloc4 4` per slot, then a
line or two per instruction. `Ty` maps to `w`; `Un::Not` prints as
`ceqw x, 0`. The `$main` stub is emitted here unchanged in shape, now calling
`$.Mod.init`. No `Scope`, no `resolve`, no AST imports remain.

### src/driver.rs

`sema::analyze` now yields the program or diagnostics; on success the IR is
debug-traced and handed to `qbe::emit(&program)`.

### runtime/oberon.c

Add `#include <stdlib.h>` and `oberon_div_by_zero` as quoted above.

### tests/corpus.rs

Add the `tests/failures` loop: compile must succeed, the binary must exit
nonzero, and its stderr must equal `X.expected` byte for byte.

## New corpus modules

`tests/corpus/` (compile, run, compare stdout):

- `Fib.Mod` — a directly recursive function procedure; the gate's recursion
  program.
- `Swap.Mod` — a proper procedure updating two `VAR` INTEGER parameters, plus
  a BOOLEAN `VAR` parameter, called with module variables and with locals.
- `Counter.Mod` — procedures that read and write a module variable, proving
  module data outlives and is shared across activation records.
- `Params.Mod` — value parameters assigned inside the callee (§10.1) with the
  caller's actual observed unchanged; both types in every legal role
  (constant, module variable, local, value parameter, `VAR` parameter,
  result); a parameterless proper procedure called both as `P` and `P()`; the
  same `DIV`/`MOD` expressions computed as CONSTs and via variables, printed
  side by side so folding and runtime lowering are compared in one output.

`tests/errors/` (must fail with exact stderr):

- `VarActualExpr.Mod` — a constant and an expression as `VAR` actuals.
- `VarTypeMismatch.Mod` — a BOOLEAN variable passed to a `VAR` INTEGER formal.
- `MissingReturn.Mod` — a function procedure with no RETURN clause.
- `ReturnMismatch.Mod` — a RETURN expression of the wrong type.
- `ReturnInProper.Mod` — a RETURN clause in a proper procedure.
- `ProcAsValue.Mod` — a proper procedure used as a value.
- `FuncAsStatement.Mod` — a function procedure called as a statement.
- `NestedProc.Mod` — pins the clean "not yet supported" diagnostic until
  Slice 5 replaces it.

`tests/failures/` (new class: compile, run, nonzero exit, exact stderr):

- `DivZero.Mod` — `x DIV y` with `y = 0` read from a variable.
- `ModZero.Mod` — same for `MOD`.

## Verification

1. Mid-slice milestone: after the IR migration but before any procedure
   support, the existing corpus and error suites must be green. This is the
   roadmap's completion criterion — every existing program passes through the
   typed IR with no direct AST-to-QBE path left — checked before the new
   feature lands on top.
2. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
   `cargo test` at the end.
3. Inspect generated IL, since this slice changes storage, calls, and control
   flow: `build/Counter.ssa` for module variables as `data` objects,
   `build/Fib.ssa` for the recursive call and the per-site division traps,
   `build/Swap.ssa` for `l` parameters and address arguments.
4. Grep qbe.rs for `resolve`, `Scope`, and `ast::` — all three must be gone.
5. Run the gate binaries by hand once (`./build/Fib`, `./build/Swap`,
   `./build/Counter`, `./build/DivZero; echo $?`) to see real output and the
   real failure behavior.

## Order of work

1. `ir.rs` — the data types alone.
2. Rework sema into check-and-lower and qbe.rs into a printer for the existing
   feature set, moving short-circuit and floored DIV/MOD into lowering. Full
   existing suite green (milestone above).
3. ast.rs and parser.rs — procedure declarations, formal parameters, call
   expressions.
4. sema.rs — procedure symbols, local scopes, parameters, RETURN rules, call
   checking and lowering for both statement and expression calls.
5. Zero checks, `oberon_div_by_zero`, and the `tests/failures` harness class.
6. Gate modules and `.expected` files; regression modules for anything found
   on the way.
