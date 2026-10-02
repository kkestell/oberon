# Slice 2: Walking skeleton — Oberon source in, native executable out

## Context

Slice 1 proved the back half: [driver.rs](../../src/driver.rs) writes a hardcoded QBE IL const to `build/out.ssa`, runs `qbe`, then one `cc` invocation assembles + compiles [runtime/oberon.c](../../runtime/oberon.c) + links `-lgc`. Slice 2 builds the front half (lexer → parser → sema → QBE emission) and connects it, so every later slice is "add a language feature" rather than "build a stage." Per explicit decision, **no typed IR this slice** — AST → QBE IL directly, with a `// TODO` citing the architecture doc; the IR arrives in slice 3/4 when there's real lowering (addresses, type descriptors, bounds checks) to justify it.

Target program `tests/corpus/Arith.Mod` (expected stdout `22\n`):

```
MODULE Arith;
  IMPORT Out;
  CONST N = 7;
  VAR x, y: INTEGER;
BEGIN
  x := N * 6;
  y := x DIV 2 + 1;
  Out.Int(y, 0); Out.Ln
END Arith.
```

## New files and shapes

### `src/diag.rs` (~15 lines)

Shared by lexer/parser/sema, so it gets its own tiny file:

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pos { pub line: u32, pub col: u32 }   // 1-based

#[derive(Debug, Clone)]
pub struct Diagnostic { pub pos: Pos, pub msg: String }
```

### `src/lexer.rs` — complete Oberon-07 token set

Flat `enum Tok` (no `Keyword(Kw)` sub-enum — keeps `expect(Tok::Semi)` a plain `PartialEq` compare):
- Literals: `Ident(String)`, `Int(i64)` (decimal and `0FFH` hex — hex starts with a digit per grammar line 8; i64 so big hex lexes, sema range-checks to i32), `Real(f64)` incl. ScaleFactor, `Char(u8)` (the `41X` form, grammar line 12), `Str(String)` (no escape sequences in Oberon).
- All 32 keywords as individual variants (ARRAY…WHILE), all operators/delimiters (`:=`, `..`, `#`, `<=`, `>=`, `^`, `~`, `&`, `|`, braces/brackets, …), `Eof`.
- `pub struct Token { pub tok: Tok, pub pos: Pos }`; `pub fn lex(src: &str, diags: &mut Vec<Diagnostic>) -> Vec<Token>` — eager full Vec terminated by `Eof`.
- Nested `(* … *)` comments via depth counter. Unterminated comment/string, illegal char → push Diagnostic, skip, continue.
- Tricky spot: while scanning digits, `1..2` vs `1.5` — peek two chars; `digit "." "."` ends the integer and leaves `DotDot`.
- `#[cfg(test)]` unit tests (the one place AGENTS.md blesses them): decimal/hex ints, reals + scale factor, char literal, string, `1..2` disambiguation, nested comment, unterminated-string diagnostic, keyword-vs-ident.

### `src/ast.rs` — subset, `Pos` inline on diagnostic-able nodes

```rust
pub struct Module { pub name: String, pub pos: Pos, pub imports: Vec<Import>,
                    pub consts: Vec<ConstDecl>, pub vars: Vec<VarDecl>, pub body: Vec<Stmt> }
pub struct Import { pub name: String, pub alias: Option<String>, pub pos: Pos }
pub struct ConstDecl { pub name: String, pub pos: Pos, pub expr: Expr }
pub struct VarDecl { pub names: Vec<(String, Pos)>, pub ty: Designator }  // TODO: enum when StrucType lands

pub struct Designator { pub ident: String, pub selectors: Vec<Selector>, pub pos: Pos }
pub enum Selector { Field(String, Pos) }  // TODO: Index, Deref, TypeGuard

pub enum Stmt {
    Assign { lhs: Designator, rhs: Expr, pos: Pos },
    Call { proc: Designator, args: Vec<Expr>, pos: Pos },
}
pub enum Expr {
    Int { value: i64, pos: Pos },
    Name(Designator),
    Unary { op: UnOp, expr: Box<Expr>, pos: Pos },
    Binary { op: BinOp, lhs: Box<Expr>, rhs: Box<Expr>, pos: Pos },
}
pub enum UnOp { Neg }   // unary + dropped in parser
pub enum BinOp { Add, Sub, Mul, Div, Mod }
```

Key generalization: `Out.Int` parses as `Designator { ident: "Out", selectors: [Field("Int")] }` — the parser cannot distinguish module qualification from field selection (grammar lines 37, 53–54), so **sema** decides. This is how qualified-ident resolution gets built once, not special-cased. Small `fn pos(&self) -> Pos` on `Expr` for sema.

### `src/parser.rs` — recursive descent, fail-fast

`struct Parser { toks: Vec<Token>, i: usize }` with `peek/advance/expect/expect_ident`. Returns `Result<Module, Diagnostic>` — **fail-fast on first parse error** (sync-and-continue is speculative robustness; multi-diagnostic collection happens in sema, which walks a valid AST — note the asymmetry in a one-line comment).

Implemented: module header + trailing `END ident "."` name check (mismatch = real diagnostic, it's a user error), ImportList incl. `:=` alias, CONST section, VAR section (type as plain designator), StatementSequence (assignment, call), expression/SimpleExpression/term/factor precedence for `+ - * DIV MOD`, unary `-`, parens, integer literals, designators with `.ident` selectors.

Everything else — TYPE/PROCEDURE sections, `/` `&` `OR` `~`, relations, IF/WHILE/REPEAT/FOR/CASE, sets, strings/NIL/TRUE/FALSE in factor, `[ ]`/`^`/type-guard selectors — is `todo!("...")`.

### `src/sema.rs` — flat scope, folding, INTEGER-only checking

```rust
pub type Scope = HashMap<String, Symbol>;
pub enum Symbol {
    Const(i32),                                   // folded at declaration
    Var,                                          // INTEGER-only this slice
    Proc { runtime_name: String, arity: usize },  // e.g. "oberon_out_int"
    Module(HashMap<String, Symbol>),              // "Out" pseudo-scope
    TypeName,                                     // pre-seeded "INTEGER"
}
pub fn analyze(module: &ast::Module) -> (Scope, Vec<Diagnostic>)
```

- `IMPORT Out` inserts `Symbol::Module(out_scope())` under the name/alias; `out_scope()` = `{"Int": Proc{oberon_out_int, 2}, "Ln": Proc{oberon_out_ln, 0}}`. Unknown module → diagnostic.
- Designator resolution (one function, reused by codegen): look up base ident; if `Module` and first selector is `Field`, step into member map; other selector use → `todo!()`; undeclared at either step → diagnostic.
- Const folding at declaration, in order (so `CONST A=2; B=A*3` works): recursive `eval_const` with `checked_*` on **i32**; overflow, const DIV/MOD by zero, and out-of-i32-range literals → diagnostics. Use `div_euclid`/`rem_euclid` — matches Report 8.2.2 for `y > 0`.
- Checks: assignment LHS is `Var`; expressions are INTEGER (Module/Proc/TypeName used as value → diagnostic); call target is `Proc` with matching arity; args INTEGER.
- Output is just `(Scope, Vec<Diagnostic>)` — no annotated AST (that's the deferred typed IR; don't build half of it now). Codegen re-resolves against the scope with `expect("sema resolved this")`.

### `src/qbe.rs` — AST → IL

`pub fn emit(module: &ast::Module, scope: Scope) -> String`; `struct Gen { out: String, scope: Scope, tmp: usize }`, `gen_expr -> String` returning an operand (`"7"` or `"%.t3"`). Top-of-file `// TODO: typed IR between sema and here (see AGENTS.md architecture)`.

- Temps `%.t0, %.t1, …` — leading `.` is legal in QBE and can't collide with Oberon idents, so VARs use source names `%x`, `%y`.
- Mangling `$<Module>_<name>` (underscore can't appear in Oberon idents) → `$Arith_init`.
- Module VARs: `alloc4 4` slots in `$Arith_init`'s `@start` + `// TODO: must become data globals once procedures reference them`.
- DIV/MOD → QBE `div`/`rem` + `// TODO: truncating; Report 8.2.2 wants floored for x < 0`. Unary minus → `neg`.
- **`$main` is emitted by qbe.rs** (it's generated code — depends on module name): `call $oberon_init()`, `call $Arith_init()`, `ret 0`.

Expected IL for Arith.Mod (N already folded to 7 by sema):

```
function $Arith_init() {
@start
	%x =l alloc4 4
	%y =l alloc4 4
	%.t0 =w mul 7, 6
	storew %.t0, %x
	%.t1 =w loadw %x
	%.t2 =w div %.t1, 2
	%.t3 =w add %.t2, 1
	storew %.t3, %y
	%.t4 =w loadw %y
	call $oberon_out_int(w %.t4, w 0)
	call $oberon_out_ln()
	ret
}
export function w $main() {
@start
	call $oberon_init()
	call $Arith_init()
	ret 0
}
```

### `runtime/oberon.c` additions

Oakwood `Out.Int(i, n)`: right-justified, space-padded, minimum field width `n`; `printf("%*d", …)` is exactly that (`n=0` → bare digits). `int32_t` matches QBE `w` in the C ABI.

```c
#include <stdint.h>
#include <stdio.h>

void oberon_out_int(int32_t v, int32_t n) { printf("%*d", (int)n, (int)v); }
void oberon_out_ln(void)                  { putchar('\n'); }
```

### Driver rewrite ([src/driver.rs](../../src/driver.rs), [src/main.rs](../../src/main.rs))

- `main.rs`: `mod diag; mod lexer; mod ast; mod parser; mod sema; mod qbe; mod driver;` (no lib.rs — bin crate stays); `std::env::args().nth(1)` for the source path; missing → `usage: oberon <file.Mod>` to stderr, exit 2. No clap, no flags; `--emit-il` stays a `// TODO`.
- `driver::build(source: &Path)`: read (anyhow + context) → lex → parse (parse `Err` pushes its diagnostic) → stop if diags → `sema::analyze` → stop if diags → `qbe::emit` → write `build/<Module>.ssa` → existing `run()` helper for `qbe` and the single `cc … runtime/oberon.c -lgc` invocation (keep recompiling the 11-line runtime every build).
- Diagnostics to **stderr** as `path:line:col: message` via `eprintln!`, then `bail!("{n} error(s)")` → exit 1.
- Output naming **`build/<ModuleName>`** — this is also what makes corpus tests parallel-safe for free.

### Test harness — `tests/corpus.rs` + corpus

One `#[test]` looping over `tests/corpus/*.Mod` (not one test per file — collect failures into a Vec, assert empty with joined message):
- Invoke the built binary via `env!("CARGO_BIN_EXE_oberon")` with `.current_dir(env!("CARGO_MANIFEST_DIR"))` (pins the driver's relative `runtime/oberon.c` and `build/` paths).
- Assert compile exit 0 (forward stderr into the failure message), run `build/<stem>`, compare stdout **bytes** to sibling `<stem>.expected`.
- Corpus: `Arith.Mod` + `Arith.expected` (`22\n`), plus one or two cheap extras (CONST-referencing-CONST, unary minus → negative output) — each costs only a `.Mod`/`.expected` pair.

## Ordered stages (each leaves `cargo build && cargo test` green)

1. `diag.rs` + `lexer.rs` with unit tests; register mods. Driver untouched.
2. `ast.rs` + `parser.rs`; driver takes the source-path arg, lexes/parses, prints diagnostics — but still emits the hardcoded IL. `tracing::debug!("{module:#?}")` dumps the AST.
3. `sema.rs`; driver reports sema diagnostics; output still hardcoded IL.
4. Runtime additions + `qbe.rs`; driver swaps hardcoded `IL` for `qbe::emit(...)` and `build/<Module>` naming. Verify: `cargo run -- tests/corpus/Arith.Mod && ./build/Arith` → `22`.
5. `tests/corpus/` + `tests/corpus.rs`; `cargo test` runs lexer units + end-to-end corpus.

## Verification

- `cargo test` — lexer literal unit tests + corpus harness (compile, run, diff stdout).
- Manual: `cargo run -- tests/corpus/Arith.Mod && ./build/Arith` prints `22`; `RUST_LOG=debug` shows tool command lines and AST dump on stderr.
- Negative checks by hand: an undeclared identifier and a wrong-arity `Out.Int(y)` each produce a `path:line:col: message` diagnostic and exit 1.

## Temptations declined (per AGENTS.md)

Typed IR / annotated AST (explicitly deferred, TODO'd) · `Spanned<T>` wrappers · parser error recovery · `Keyword(Kw)` sub-enum or lexer iterator · lib.rs split · per-file generated tests/tempdirs · correct floored DIV/MOD for negative operands (cited TODO; folder already euclidean).
