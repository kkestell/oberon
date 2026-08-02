# References

## Language definition

[references/oberon07-grammar.ebnf](../../references/oberon07-grammar.ebnf)
: EBNF grammar for Oberon-07, rearranged from the Report proper (not the Appendix), including the Project Oberon 2013 change making `CASE` a type guard for record extension.

[references/oberon07-report.pdf](../../references/oberon07-report.pdf)
: Wirth, *The Programming Language Oberon (Revised Oberon-07)*, May 2016. The normative language definition.

## Reference implementations

Vendored under `references/compilers/` for consultation when behaviour is
ambiguous. These are for **reading**, not for copying — OBNC in particular is
GPL-3.0 and must not be lifted into this tree.

[references/compilers/obnc](../../references/compilers/obnc) — [miasap.se/obnc](https://miasap.se/obnc/) (0.17.2, GPL-3.0)
: Oberon-07 to C translator, and the closest analogue to this compiler: it also lowers to a low-level target over BDWGC, so it has to make record layout, type descriptors, dynamic type tests, open-array length passing, and module init order explicit. Consult `src/Types.c` for type compatibility, `src/Generate.c` for lowering, and `lib/obnc/` for the runtime split.

[references/compilers/project-oberon](../../references/compilers/project-oberon) — [projectoberon.com](http://www.projectoberon.com/)
: Wirth's own compiler — the authority on what the language actually means. `ORS.Mod` scanner, `ORP.Mod` parser, `ORB.Mod` symbol table and import/export, `ORG.Mod` RISC5 code generator. Single-pass with no AST or IR, so it is a reference for *what* is correct, not for how to structure this compiler.

[references/compilers/oberonc](../../references/compilers/oberonc) — [github.com/lboasso/oberonc](https://github.com/lboasso/oberonc) (MIT)
: Oberon-07 to JVM bytecode, written in Oberon. `OJP`/`OJB`/`OJG.Mod` follow Wirth's parser/symbol-table/generator split. Useful for scope and symbol-table handling; less so for lowering, since the JVM supplies GC, bounds checks, and object layout.
