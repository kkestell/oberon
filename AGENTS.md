# Oberon-07 Compiler

A small, native Oberon-07 compiler written in Rust. The project prioritizes simplicity, correctness, readability, and faithfulness to the language over aggressive optimization or broad platform support.

## Goals

* Implement the complete Oberon-07 language.
* Produce native executables.
* Keep the compiler small and easy to understand.
* Favor straightforward implementations over clever ones.
* Build on well-established components rather than reinventing infrastructure.

## References

The Report and its grammar are the normative definition of the language, and three other Oberon compilers are vendored under `references/` to consult when behaviour is ambiguous. See [docs/dev/references.md](docs/dev/references.md) for what each one is good for.

## English, Please

You compress. The reader decompresses. Stop outsourcing that work.

Your explanations cross three thought threads in one clause and assume the reader followed all of them. They didn't. They saw your messages — not your thinking — and they weren't taking notes on your vocabulary.

### The rules

1. **One idea per sentence.** If a clause leans on two things you established earlier, that's at least two sentences, and each thing gets re-introduced. "Its birth is the fix for both open soundness bugs and the file-origin defect" is three sentences wearing a trench coat.
2. **Your labels aren't shared vocabulary.** "The substrate", "the open stack", "Option A" — you coined those. When you reuse a coined term, re-anchor it in a few words: "the substrate (the shared statement-walking layer)". If it last appeared more than a couple messages ago, treat it as brand new.
3. **No notation in prose.** No arrow chains ("module name → File"). No hyphen-stacked compounds ("degrade-to-Partial escape-hatch"). No bold-label headers that carry the whole argument. Write the actual sentence.
4. **Don't cite reasoning the reader never saw.** "As I noted", "this one I under-stated before" — if it happened in your thinking, or more than a few messages back, restate it. Don't footnote it.
5. **Short vs. clear: choose clear.** Density that's recoverable with close reading is still a failure. The reader shouldn't need to read closely.

### The test

Reread your draft as someone who skimmed the visible messages and took no notes. Every place they'd have to stop and reconstruct — that's your job, not theirs. Unpack it.

## Architecture

```text
Source
    ↓
Lexer
    ↓
Parser
    ↓
AST
    ↓
Semantic Analysis
    ↓
Typed IR
    ↓
QBE IL
    ↓
QBE
    ↓
System Assembler + Linker
    ↓
Native Executable
```

The compiler uses a small typed intermediate representation between semantic analysis and code generation. The IR makes addresses, values, storage, calls, and runtime operations explicit while remaining much smaller than LLVM IR.

## Backend

The backend targets QBE.

QBE provides:

* instruction selection
* register allocation
* calling conventions
* native code generation

The compiler is responsible for language semantics, object layout, runtime checks, and lowering to QBE IL.

## Runtime

A small C runtime provides:

* memory allocation
* module initialization
* runtime checks
* basic runtime support
* standard library implementation

The runtime intentionally remains minimal.

## Garbage Collection

Heap allocation uses the Boehm–Demers–Weiser conservative garbage collector (BDWGC).

Generated code allocates through runtime wrappers rather than calling BDWGC directly:

* `oberon_alloc`
* `oberon_alloc_atomic`

This isolates the compiler from the underlying allocator and allows the GC implementation to be replaced in the future if desired.

## Implementation Strategy

Develop incrementally.

1. Lexer
2. Parser
3. AST
4. Semantic analysis
5. Typed IR
6. QBE code generation
7. Runtime
8. Standard modules
9. Conformance and regression testing

Each stage should produce a working compiler capable of compiling progressively more of the language.

## Plans and Reviews

Development happens in slices. Every slice gets a plan written before the work starts, in [docs/dev/plans/](docs/dev/plans/), and every review gets a document written as it is delivered, in [docs/dev/reviews/](docs/dev/reviews/). Do this automatically, without being asked. See [docs/dev/plans-and-reviews.md](docs/dev/plans-and-reviews.md) for the file naming scheme and what each kind of document should contain.

## Non-Goals

* Self-hosting
* LLVM
* JIT compilation
* Advanced optimization
* IDE features
* Incremental compilation
* Language extensions

The focus is a clean, faithful, native Oberon-07 compiler with a small, understandable implementation.

## Code Style

The rule is "Just Enough" Rust: this is an experiment, so optimize for code that is cheap to change rather than robust to operate. Oberon-07 semantics deserve care and faithfulness to the Report; the Rust implementing them should stay thin and boring. Read [docs/dev/code-style.md](docs/dev/code-style.md) before writing code — it covers errors and diagnostics, ownership, when an abstraction is allowed, file structure, and testing.

