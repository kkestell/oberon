# Architecture

```text
Root source file
    ↓
Module discovery  ──→  for each module: Lexer → Parser → AST → Semantic Analysis
    ↓
Aggregate typed IR
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

## Modules

A build starts from one root source file and compiles every module that file reaches through imports. Imports are followed depth first, in the order they are written, and a module reached twice is compiled once. An import naming a module whose compilation has already started is an import cycle and ends the build.

Each module is compiled completely before any module that imports it. Analysis therefore returns two things: the module's IR and its interface. The interface holds only the declarations carrying an export mark, so a client importing the module can see nothing else.

### Source lookup

The root source file's directory is the build's root directory. An import of module `M` looks for a file named exactly `M.Mod` in two places, in this order:

1. The root directory.
2. The bundled `lib/` directory.

The root directory comes first so an application can supply a module that shadows a bundled one. If neither directory has the file, the module name `Out` falls back to a temporary interface backed by the C runtime. That fallback is the last piece of the standard library with no Oberon source, and it goes away when `lib/Out.Mod` arrives.

Every source module must live in a file named after it. There are no search paths, packages, or serialized interfaces, and nothing is cached between invocations: each build rereads and reanalyzes every source it needs.

### Initialization

All source modules of one build are emitted into a single QBE unit, so a call from one to another needs no linkage annotation. Export marks are a semantic rule that analysis has already applied by the time code is generated.

The generated `main` calls `oberon_init` and then calls each module's initializer once, in the order the modules were compiled. Because a dependency finishes before its client starts, that order runs every module body after the bodies of everything it imports, and the root module's body last.

## Scalar representation

Every scalar type the compiler supports occupies four bytes and travels in a QBE word. `INTEGER` is a signed 32-bit number, `BOOLEAN` is 0 or 1, and `SET` is a 32-bit bit vector. The three remain distinct semantic types: sharing a machine class never makes one assignable to another.

The Report leaves the largest `SET` element implementation-defined. This compiler chooses 31, so a `SET` contains exactly the integers 0 through 31 and bit *n* records membership of element *n*. The empty set is zero and the full set is every bit. A `SET` module variable starts empty because static storage is zero-filled; a `SET` local, like every other local, is uninitialized until assigned.

`ORD` applied to a `SET` reinterprets that bit pattern as a signed `INTEGER` and emits no instruction. `ORD({0})` is 1, `ORD({31})` is `MIN(INTEGER)`, and `ORD(-{})` is -1. Constant folding performs the same reinterpretation, so a folded result and a computed one always agree.

Anything that produces or tests a `SET` element checks that element against 0 through 31 first: a constructor element, either endpoint of a range, the left operand of `IN`, and the second argument of `INCL` and `EXCL`. An element the compiler can fold is a source diagnostic, so `{16 + 16}` fails to compile exactly as `{32}` does. An element that is only known at run time is compared against both bounds before the shift that would otherwise consume it, because QBE reduces a shift count modulo the word width and would silently read 32 as 0. A failed check calls `oberon_set_element_range`, which prints one line and exits.

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
