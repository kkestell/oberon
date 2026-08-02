# Code Style

## "Just Enough" Rust

This is a hobby / experimentation project. Optimize for **cheap to change**, not robust to operate. The value of this code is how fast I can rewrite it tomorrow, so the goal is to minimize committed surface area. When in doubt, do **less**.

Note the split of concerns: *Oberon-07* semantics deserve care and faithfulness to the Report. The *Rust* implementing them should stay as thin and boring as possible.

## North star

Separate two things the word "safety" bundles together:

- **Correctness** — no UB, no data races, no type confusion. Rust gives this for free in safe code. Keep it, for free.
- **Robustness** — defending against inputs and states that may never occur (error taxonomies, fallback paths, swappable backends, config layers). Spend almost nothing here. Defensive code is a bet that the current design is right, and here the design is _expected_ to change.
- **Boundary sanity is not defensive programming.** Malformed Oberon source is ordinary input, not a bug: a truncated file, an unterminated string, a `RETURN` in a proper procedure, an undeclared identifier. Those must produce a clear diagnostic with a source position and a nonzero exit, never a panic and never a wrong binary. Broken *internal* invariants — an unresolved symbol reaching codegen, an IR node with no type, a QBE temporary used before it is defined — should crash loudly and immediately. The boundary is source text; everything past semantic analysis is our own invariant to keep.

Default to the simplest thing that compiles and reveals whether the idea works. Prefer doing less and leaving a `// TODO:` over building the hardened version speculatively. A stage that handles half the language and `todo!()`s the rest is the expected intermediate state.

## Keep — this is still real, idiomatic Rust

- **Safe Rust only** unless I explicitly ask otherwise. Lean on the compiler fully; it is the safety net that lets the rest of this file be aggressive.
- **The expressive core stays:** iterators, `?`, `Option`, pattern matching, `impl Trait` in argument position, and liberal `#[derive(Debug, Clone, Default, PartialEq, ...)]`.
- **`enum` + `match` for AST, types, and IR.** This is the one place a rich type is the simplest thing: exhaustive `match` over `Expr`, `Stmt`, and `Type` is how a missing case in a new stage gets found at compile time. Do not erase that into trait objects or `Box<dyn Any>`.
- **Abstractions the code has _earned_ by repeating.** Discovered abstractions: yes. Speculative ones imposed up front: no. Idiomatic ≠ enterprise.

## Errors

Two kinds of failure, handled differently:

**Diagnostics** — the user's Oberon program is wrong. This is the compiler's real output when it isn't emitting code, so it gets one small owned struct (position + message), collected in a `Vec` so several can be reported from one run. That is not an error taxonomy: no enum variant per rule, no error codes, no `thiserror`. The message is a `String`, written for a human reading their source.

**Everything else** — driver plumbing: reading the source file, spawning QBE, invoking the assembler and linker.

- Use `anyhow::Result<T>` and `?` for the driver and any I/O. Add `.context("...")` where it aids debugging (`.context("running qbe")`).
- `unwrap()`, `expect("invariant")`, and `panic!` are fine and often _preferred_ for compiler-internal invariants — a loud crash points straight at the broken stage, while a swallowed error yields a silently wrong executable. Treat `expect` as an executable assertion.
- Do **not** use `unwrap_or_default()` or other quiet fallbacks around parse or type results. A missing type is a bug in an earlier stage; substituting `INTEGER` to keep going hides it until codegen or, worse, until the compiled program misbehaves.
- Do **not** define custom error enums beyond the diagnostic struct unless I actually `match` on the error to recover differently. An error that is only ever printed does not need a type.

```rust
// Don't — an error taxonomy for something only ever printed
#[derive(Debug, thiserror::Error)]
pub enum SemaError {
    #[error("undeclared identifier {0}")] Undeclared(String),
    #[error("type mismatch")] TypeMismatch { expected: Type, found: Type },
    /* ...forty more... */
}

// Do
pub struct Diagnostic { pub pos: Pos, pub msg: String }

fn undeclared(pos: Pos, name: &str) -> Diagnostic {
    Diagnostic { pos, msg: format!("undeclared identifier '{name}'") }
}
```

## Ownership & lifetimes

- **Clone freely.** Prefer owned data: `String` over `&str` in struct fields, owned `Vec<T>` over borrowed slices in fields, `Rc`/`Arc` to share.
- **Avoid lifetime annotations in structs.** A `&'a str` identifier borrowed from the source buffer, or an AST node borrowing its parent, couples lifetimes virally and ossifies the design before its shape is known. Own the `String`, `Box` the child, index into a `Vec` with a plain `usize` id. Tighten later only if a measured hot path demands it.
- Don't micro-optimize allocations or chase zero-copy. A whole Oberon module is a few thousand lines; clear and correct beats clever, and the compiler will still be effectively instant.

## Abstraction

- **Concrete types until the rule of three** (two real implementations plus a third in sight). No traits and no generics for a single caller.
- There is **one** backend (QBE) and **one** allocator (BDWGC). Do not introduce a `trait Backend` or a `trait Allocator` for them — the runtime wrappers `oberon_alloc`/`oberon_alloc_atomic` are already the seam that keeps a swap cheap, and it costs nothing until it's needed.
- No builder patterns, no plugin/registry architectures, no dependency injection. A visitor trait over the AST is the same mistake: write a plain recursive `fn` that `match`es.

```rust
// Don't — speculative generality for one caller
trait Backend { fn emit_call(&mut self, ...); /* ... */ }
fn lower<B: Backend>(ir: &Ir, backend: &mut B) { ... }

// Do
fn lower(ir: &Ir, out: &mut String) { ... }   // writes QBE IL
```

## Structure & config

- **One file, one concern.** Separation of concerns matters even under Just Enough Rust. The pipeline stages are the seams, so they get their own files: `lexer.rs`, `parser.rs`, `ast.rs`, `sema.rs`, `ir.rs`, `qbe.rs`, `driver.rs`. A clean seam is reason enough to split; you don't have to wait for a file to get painful. The aim is cohesion, not short files — `parser.rs` being long is fine, a parser split across five files is not.
- Stay lean otherwise: no deep module trees and no extra crates early. Splitting the crate into focused modules, or adding a small test-harness binary that compiles and runs the conformance corpus, ≠ speculative architecture.
- Don't fuss over visibility in a single-crate binary; `pub` or default is fine.
- **Hardcode local tuning values as `const` at the top of the file** until I ask for knobs — word size, alignment, type-descriptor layout constants, the names of the runtime entry points, the path to `qbe`. A `const` takes a second to change; a speculative target-description layer takes an afternoon to remove.
- Hand-write the lexer and the recursive-descent parser. No parser-generator or combinator crate: the Oberon-07 grammar is small, LL(1), and in [references/oberon07-grammar.ebnf](../../references/oberon07-grammar.ebnf).

## Tests, comments, tooling

- Comments explain **why** / the surprising part — especially where behaviour comes from the Report or from resolving an ambiguity against a reference compiler. Cite the source (`// Report 10.1: ...`, `// cf. obnc src/Types.c`). No doc comments restating the obvious (`/// Returns the name`).
- **The real test is end-to-end:** compile an Oberon module, run the executable, compare its output. Prefer a small corpus of `.Mod` files with expected output over unit-testing internal stage APIs that are still moving. Unit tests are worth it for things with tricky, stable rules — literal scanning, type compatibility, `SET` constant folding.
- Every bug that gets fixed earns a regression module in the corpus.
- Use `tracing` for compiler internals: token streams, scope contents, IR dumps, and the exact `qbe`/assembler/linker command lines. Diagnostics and traces go to **stderr** and stay filterable; keep stdout for actual requested output (e.g. `--emit-il`).
- Keep dependencies few. Reach for `anyhow`; don't pull a crate to abstract something used once.

## When to graduate

When I say the experiment has stuck and the code is going to live, **then** harden — and harden against the failures we actually observed, not imagined ones: introduce typed errors where recovery genuinely matters, tighten lifetimes on measured hot paths, extract the abstractions that actually recurred, and grow the conformance corpus around the now-stable behaviour. Until I say so, stay lean.
