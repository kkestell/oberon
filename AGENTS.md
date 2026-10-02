# Oberon-07 Compiler

> Make things as simple as possible, but not simpler.

A small, native Oberon-07 compiler written in Rust that values simplicity, correctness, readability, and faithfulness to the language.

## Goals

* Implement the complete Oberon-07 language.
* Produce native executables.
* Keep the compiler small and easy to understand.
* Favor straightforward implementations over clever ones.
* Build on established components rather than reinventing infrastructure.

## References

The Report is Wirth's [*The Programming Language Oberon (Revised Oberon-07)*, May 2016](references/oberon07-report.pdf), the normative definition. The [extracted EBNF](references/oberon07-grammar.ebnf) gathers its grammar in one place.

When the Report is ambiguous, consult the vendored compilers. Read their code; do not copy it.

| Compiler | Consult for |
| --- | --- |
| [OBNC 0.17.2](references/compilers/obnc) (GPL-3.0) | The closest analogue: a C target with Boehm GC. `src/Types.c` for compatibility, `src/Generate.c` for lowering, `lib/obnc/` for the runtime. |
| [Project Oberon](references/compilers/project-oberon) | Wirth's own interpretation: `ORS`, `ORP`, `ORB`, and `ORG.Mod`. |
| [oberonc](references/compilers/oberonc) (MIT) | Scope and symbols in `OJP`, `OJB`, and `OJG.Mod`. The JVM supplies layout, bounds checks, and GC, so its lowering compares poorly. |

[docs/about-oberon-07.md](docs/about-oberon-07.md) introduces the language to a reader new to Oberon; it explains and defers to the Report. The Report defines no standard library; [docs/standard-library.md](docs/standard-library.md) specifies the bundled modules in [lib/](lib/). [examples/](examples/) holds attributed programs from other Oberon projects; nothing in the build depends on them.

## Architecture

```text
Source → Lexer → Parser → AST → Semantic analysis → Typed IR
       → QBE IL → QBE → System assembler and linker → Executable
```

The compiler owns semantics, object layout, and runtime checks. QBE owns instruction selection, register allocation, and calling conventions. QBE 1.3 and the collector are [vendored](vendor/README.md), so a C compiler is the only host requirement.

The [driver](src/driver.rs) compiles one reachable module graph into one QBE unit. Dependencies finish before their clients, which yields both the interfaces for analysis and the order of initialization. Interfaces share type descriptors, so type identity survives imports. Nothing persists between builds.

[Semantic analysis](src/sema/) reduces source to a small [typed IR](src/ir.rs) in which evaluation order, addresses, layout, runtime checks, and hidden arguments are explicit. [QBE lowering](src/qbe.rs) only translates.

### Representation

`INTEGER` is signed 32-bit, `REAL` is IEEE binary32, `SET` is a 32-bit vector, and `CHAR` and `BYTE` are one byte. `SYSTEM` is not implemented.

Structured parameters are passed by address. The Report requires copies only for basic value parameters (10.1) and forbids modifying structured ones (9.1), so no copy is needed; aliasing through a `VAR` parameter stays observable. Open arrays carry their lengths, and record `VAR` parameters carry their dynamic type.

Procedure values are code addresses. Only module-scope procedures can be values, so there are no closures.

An extended record contains its base as a prefix. Static descriptors link each record type to its base, and heap objects carry their descriptor in a hidden header, leaving field offsets and copy sizes unchanged.

### Runtime

[runtime/oberon.c](runtime/oberon.c) supplies language checks, predefined operations, and allocation through the Boehm–Demers–Weiser collector. Interior pointers keep objects alive; pointer-free payloads are unscanned. The runtime and the collector are prebuilt into one static `liboberon.a`, which every program links.

The [standard modules](lib/) are ordinary Oberon source. [runtime/standard.c](runtime/standard.c) provides I/O, formatting, libm, and operating-system access through a private interface visible only to bundled modules. Only scalars, bounded buffers, and opaque file handles cross it. Language failures terminate the program; fallible library operations return status.

Interpretation decisions are recorded in [agents/reviews/](agents/reviews/).

### Building

The compiler finds its support directory, holding `qbe`, `liboberon.a`, and the bundled modules, at `../lib/oberon` from its own executable. `make` builds the release compiler and stages that directory in `target/lib/oberon/`, which the debug and release compilers share. `make test` stages it and runs `cargo test`; a plain `cargo test` uses whatever `make` last staged, so runtime C and `lib/*.Mod` edits need `make` first. `make install [PREFIX=...]` installs `bin/oberon` and `lib/oberon/` under `PREFIX`, by default `~/.local`.

## Code Style

"Just Enough" Rust. This is an experiment: optimize for code that is cheap to change, not robust to operate. The Oberon semantics deserve care; the Rust should stay thin and plain. Harden against failures once they are observed.

### Errors

* Malformed or unsupported source produces a positioned diagnostic and a nonzero exit. Diagnostics are position-and-message values collected in a `Vec` and printed in source order.
* A broken invariant fails loudly with `expect`, `unwrap`, or `panic!`. Never guess or recover silently.
* I/O and driver failures use `anyhow` with context. Add an error type only when callers recover differently.

### Implementation

* Safe Rust and ordinary idioms, unless told otherwise.
* AST, types, and IR are enums, matched exhaustively.
* Own data and clone freely; use indices and shared handles rather than lifetimes. Optimize only after measuring.
* Stay concrete until two implementations exist and a third is in sight.
* Modules follow pipeline stages; distinct subpasses may use shallow directories. Length alone is no reason to split a file.
* Constants, not configuration. One backend, one allocator.
* The lexer and recursive-descent parser are written by hand.

### Testing and Comments

* Prefer end-to-end corpus tests that compile, run, and compare output or diagnostics. Unit-test only tricky, stable rules.
* Comments explain why, citing the Report or a reference compiler where it helps.
* Compiler internals log through `tracing`. Diagnostics and traces go to stderr; stdout is for requested output.
