# Oberon-07 Compiler

> Make things as simple as possible, but not simpler.

A small, native Oberon-07 compiler written in Rust. The project prioritizes simplicity, correctness, readability, and faithfulness to the language over aggressive optimization or broad platform support.

## Goals

* Implement the complete Oberon-07 language.
* Produce native executables.
* Keep the compiler small and easy to understand.
* Favor straightforward implementations over clever ones.
* Build on well-established components rather than reinventing infrastructure.

## References

The normative language definition is Niklaus Wirth's *The Programming Language Oberon (Revised Oberon-07)*, May 2016, referred to here as the Report. An extracted EBNF grammar is kept alongside it for convenient reference. Three Oberon compilers are also vendored as secondary references for cases where the language definition is ambiguous. See [docs/dev/references.md](docs/dev/references.md) for the language definition files, the reference implementations, and guidance on when to consult each one.

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

See [docs/dev/architecture.md](docs/dev/architecture.md) for the compiler pipeline, backend, runtime, and garbage collector.

## Code Style

The rule is "Just Enough" Rust: this is an experiment, so optimize for code that is cheap to change rather than robust to operate. Oberon-07 semantics deserve care and faithfulness to the Report; the Rust implementing them should stay thin and boring. Read [docs/dev/code-style.md](docs/dev/code-style.md) before writing code — it covers errors and diagnostics, ownership, when an abstraction is allowed, file structure, and testing.

## Plans and Reviews

Development happens in slices. Every slice gets a plan written before the work starts, in [docs/dev/plans/](docs/dev/plans/), and every review gets a document written as it is delivered, in [docs/dev/reviews/](docs/dev/reviews/). Do this automatically, without being asked. See [docs/dev/plans-and-reviews.md](docs/dev/plans-and-reviews.md) for the file naming scheme and what each kind of document should contain.

Only write plans and do reviews for major slices of work. Not for small tasks, documentation updates, or minor bug fixes. If asked to review a plan, do not write a review. Just give feedback on the plan.