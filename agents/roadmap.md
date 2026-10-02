# Development roadmap

The next milestone is demonstrated conformance to the
[May 2016 Oberon-07 Report](../references/oberon07-report.pdf).

## Completion criteria

Every grammar production, predefined identifier, type compatibility rule,
selector, operator, statement, parameter mode, and visibility rule needs test
coverage. Release requires no unexplained gaps or rejection of valid core
Oberon-07 programs.

Correct programs must compile without internal panics or backend errors.
Malformed source must receive positioned diagnostics and a nonzero exit.
Dynamic failures must have stable diagnostics and a nonzero exit. Constant
and runtime evaluation must agree.

Coverage includes cross-module behavior, each type's legal roles, library
boundaries, initialization, recursion, deep nesting, aggregate copies, and GC
retention. Implementation-defined choices and known limitations must be
documented.

## Scope

Support the current QBE target and native C toolchain. Optional `SYSTEM`
requires an address representation that accounts for 32-bit integers and
64-bit addresses. Interactive input and graphics are outside the portable
library profile.

Additional targets, interface caching, incremental compilation, self-hosting,
a JIT, and advanced optimization remain non-goals unless project goals change.

## Real-world use

These gaps keep the compiler from building ordinary programs outside this
repository. They follow conformance.

- **Runtime failure positions.** Runtime failures print a fixed message with no
  source position. Report the module, line, and column of the failing check.
- **Output ordering.** A runtime failure writes to stderr before buffered
  stdout is flushed, so the failure appears ahead of earlier output. Flush
  stdout before writing a failure.
- **Debug information.** QBE emits no line information, so a debugger cannot
  map generated code to source.
- **Host access.** No `SYSTEM` module or binding mechanism exists, so programs
  reach the operating system only through the bundled modules. Define `SYSTEM`
  or another way to bind C functions.
- **Numeric width.** `INTEGER` is 32-bit and `REAL` is binary32. Array lengths,
  indices, and file positions share the 32-bit limit, and `INTEGER` overflow
  wraps without a trap.
- **Check cost.** Every index, nil, and range check is a runtime call. Inline
  the passing comparison and call the runtime only on failure.
- **Emitted IL.** `--emit-il` is unimplemented.

Two limitations remain under the current scope: the compiler supports one
target, and every build reanalyzes the whole module graph with no separately
compiled modules.
