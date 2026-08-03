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

Every scalar type the compiler supports occupies four bytes. `INTEGER` is a signed 32-bit number, `BOOLEAN` is 0 or 1, and `SET` is a 32-bit bit vector; all three travel in a QBE word. `REAL` is an IEEE 754 binary32 value and travels in a QBE single, which is the platform's native C `float` calling class. The four remain distinct semantic types: sharing a machine class never makes one assignable to another, and `INTEGER` and `REAL` never mix implicitly even though both are numeric.

### REAL

Project Oberon gives `REAL` four bytes and operates on the binary32 sign, exponent, and fraction fields. QBE defines its `s` type as an IEEE 754 32-bit float and passes it in the platform's floating-point class. The two agree, so no compiler-specific calling convention is needed and no target description or configurable floating-point layer exists.

A decimal literal is parsed straight to binary32 in the lexer, so the value the AST carries is the value the generated code uses and nothing rounds twice. A literal whose magnitude has no finite binary32 representation is a source diagnostic; one too small to represent rounds to zero, which is the ordinary IEEE result.

Arithmetic rounds at binary32 at every source operator, and constant folding uses the same `f32` operations at the same points, so a folded expression and the same expression computed at run time agree. `REAL` arithmetic is plain IEEE 754: overflow yields an infinity, division by zero yields an infinity or a NaN, and neither traps, because the Report defines no trap for them. A NaN is unequal to every value including itself, and every ordering comparison involving one is false. Source has no spelling for an infinity or a NaN; both only arise from arithmetic.

A `REAL` immediate reaches QBE as the signed decimal spelling of its 32-bit pattern. A QBE constant is an untyped bit string, so in an `s` context that reproduces every binary32 value exactly, including negative zero and the non-finite values constant folding can produce.

A `REAL` module variable is a four-byte zero-filled data object, so its initial value is positive zero. A `REAL` local, like every other local, is uninitialized until assigned. A `REAL` value parameter is a writable local copy, and a `REAL` variable parameter is an address and uses the QBE pointer class.

### REAL operations

`ABS` accepts an `INTEGER` or a `REAL` and returns the argument's type. The `INTEGER` form keeps its overflow trap at `MIN(INTEGER)`; the `REAL` form calls `oberon_abs_real`, which clears the sign of a negative zero, leaves an infinity alone, and returns a NaN for a NaN.

`FLT` is the explicit `INTEGER`-to-`REAL` conversion and is the only conversion between machine classes the compiler has, so it emits QBE `swtof` and needs no general conversion matrix. It rounds to the nearest representable value and does not promise that every `INTEGER` survives: `FLT(MAX(INTEGER))` is `2147483648.0`. No assignment or arithmetic context inserts it; every conversion is written in the source.

`FLOOR` yields the largest `INTEGER` not greater than its argument, rounding toward negative infinity, so `FLOOR(-1.5)` is -2. A result exists only when the rounded argument is finite and lies from `-2147483648.0` inclusive to `2147483648.0` exclusive. A constant outside that domain is a source diagnostic. A dynamic one calls `oberon_floor`, which checks finiteness and both endpoints itself rather than relying on a float-to-integer conversion no target has to define, and which prints `FLOOR result is outside INTEGER range` and exits. The domain applies after binary32 rounding, so `FLOOR(FLT(MAX(INTEGER)))` is out of range.

`PACK(x, n)` replaces `x` with `x * 2^n` through `oberon_pack`, a wrapper around `ldexpf`. Overflow, subnormal results, and underflow to zero are ordinary binary32 outcomes and do not trap.

`UNPK(x, n)` is its inverse: it preserves the sign of `x`, normalizes the magnitude into 1 through 2 excluding 2, and stores the base-two exponent in `n`. The Report states the interval without resolving a negative argument, so this compiler follows OBNC and applies the interval to the absolute value. Project Oberon instead folds the sign into an encoded exponent, which round-trips but is not the mathematical exponent `PACK` uses, so that target-specific encoding is not adopted here. The runtime uses `frexpf`, doubling the fraction and subtracting one from the exponent because C normalizes to one half instead of one.

Zero satisfies no normalization interval and the references give no shared answer for it. `UNPK` of either zero stores positive zero in `x` and zero in `n`, which round-trips through `PACK` and invents no exponent. `UNPK` of an infinity or a NaN cannot produce a normalized value: it prints `UNPK argument is not finite` and exits.

Both arguments of `PACK` and `UNPK` are resolved left to right and evaluated once. Imported variables stay read-only, so neither procedure can reach one.

### SET

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
