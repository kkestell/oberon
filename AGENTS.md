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

[docs/about-oberon-07.md](docs/about-oberon-07.md) introduces the language itself, for a reader who has not written Oberon before. It explains rather than defines, and defers to the Report throughout.

### The test

Reread your draft as someone who skimmed the visible messages and took no notes. Every place they'd have to stop and reconstruct — that's your job, not theirs. Unpack it.

## Architecture

See [docs/dev/architecture.md](docs/dev/architecture.md) for the compiler pipeline, backend, runtime, and garbage collector.

## Code Style

The rule is "Just Enough" Rust: this is an experiment, so optimize for code that is cheap to change rather than robust to operate. Oberon-07 semantics deserve care and faithfulness to the Report; the Rust implementing them should stay thin and boring. Read [docs/dev/code-style.md](docs/dev/code-style.md) before writing code — it covers errors and diagnostics, ownership, when an abstraction is allowed, file structure, and testing.

## Plans and Reviews

Development happens in slices. Every slice gets a plan written before the work starts, in [docs/dev/plans/](docs/dev/plans/), and every review gets a document written as it is delivered, in [docs/dev/reviews/](docs/dev/reviews/). Do this automatically, without being asked. See [docs/dev/plans-and-reviews.md](docs/dev/plans-and-reviews.md) for the file naming scheme and what each kind of document should contain.

Only write plans and do reviews for major slices of work. Not for small tasks, documentation updates, or minor bug fixes. If asked to review a plan, do not write a review. Just give feedback on the plan.

Never review your own work, and never delegate that review to a subagent. If you implemented the slice, say it is ready for review and stop. Verifying the work and reporting what you observed is still yours to do.