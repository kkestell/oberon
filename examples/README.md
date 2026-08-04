# Examples

Oberon-07 programs that build and run with this compiler. They come from other
Oberon projects, and each file names its source and any change made to it in a
comment at the top. Nothing here is part of the compiler or its test suite; the
corpus in [tests/corpus/](../tests/corpus/) is what the build actually asserts.

## Building and running

Compile from the repository root, because the compiler resolves `runtime/`,
`lib/`, and `build/` relative to the working directory. The executable lands in
`build/` under the module's name.

```
./target/debug/oberon examples/Sieve.Mod
./build/Sieve
```

`build/` is flat and keyed by module name, so two modules of the same name
overwrite each other's executable.

## Programs

Each of these is one file and depends on nothing but the bundled `In` and `Out`.

| Program | What it does | Input |
| --- | --- | --- |
| [Hello.Mod](Hello.Mod) | The customary first program, in UTF-8 | |
| [Doors.Mod](Doors.Mod) | The hundred doors puzzle | |
| [Sieve.Mod](Sieve.Mod) | Sieve of Eratosthenes over 2..1000 | |
| [PrimeNumbers.Mod](PrimeNumbers.Mod) | Wirth's prime tabulation | a count in 2..99 |
| [Powers.Mod](Powers.Mod) | Wirth's positive and negative powers of two | a limit in 0..32 |
| [TempConv.Mod](TempConv.Mod) | Centigrade to Fahrenheit table | |
| [MultiplicationTables.Mod](MultiplicationTables.Mod) | Upper triangle of an 18x18 table | |
| [HeapSort.Mod](HeapSort.Mod) | Heap sort, Wirth's *Algorithms and Data Structures* 2.8 | |
| [Fact.Mod](Fact.Mod) | Factorials until the next one overflows | |
| [Exp.Mod](Exp.Mod) | The first 1024 decimal digits of *e* | |
| [Hailstone.Mod](Hailstone.Mod) | Hailstone sequences and the longest below 100000 | |
| [Sequence012.Mod](Sequence012.Mod) | Sequences with no two adjacent equal subsequences | |
| [MagicSquares.Mod](MagicSquares.Mod) | Wirth's odd-order magic squares | an odd order in 3..13 |
| [SpiralMatrix.Mod](SpiralMatrix.Mod) | Natural numbers wound inwards | a width and a height |
| [SierpinskiCarpet.Mod](SierpinskiCarpet.Mod) | The Sierpinski carpet in text | an order in 0..3 |
| [SierpinskiTriangle.Mod](SierpinskiTriangle.Mod) | The Sierpinski triangle in text | an order in 0..5 |
| [GuessNumber.Mod](GuessNumber.Mod) | Binary search for a number you picked | a name, then h/l/c |
| [Postfix.Mod](Postfix.Mod) | Infix to postfix by recursive descent | expressions, one per line |

A program that reads standard input reads it plainly, so a pipe works:

```
echo 5 | ./build/MagicSquares
printf 'a*(b/(c-d))\n\n' | ./build/Postfix
```

## Collections

[Collections/](Collections/) holds four programs that exercise dynamic data
structures over pointers, record extension, and the garbage collector. They
share one set of dependency modules, so compile them from inside that
directory's tree:

```
./target/debug/oberon examples/Collections/ExampleHeap.Mod
./build/ExampleHeap
```

| Program | What it does |
| --- | --- |
| [ExampleArrayList.Mod](Collections/ExampleArrayList.Mod) | Chunked array list with indexed access |
| [ExampleHeap.Mod](Collections/ExampleHeap.Mod) | Binary min-heap as a priority queue |
| [ExampleQueue.Mod](Collections/ExampleQueue.Mod) | FIFO queue over a linked list |
| [ExampleStack.Mod](Collections/ExampleStack.Mod) | LIFO stack driving undo/redo and postfix evaluation |

## Where these came from

The corpus these were drawn from lives in
[references/examples/](../references/examples/), which also explains what each
upstream project is. Three projects are represented:

- [AntKrotov/oberon-07-compiler](https://github.com/AntKrotov/oberon-07-compiler),
  BSD-2-Clause. Its console samples were adapted to Oberon-07 by 0CodErr of the
  KolibriOS team, and many of them are older Wirth, Modula-2, or Modula-3
  programs in turn.
- [lboasso/oberonc](https://github.com/lboasso/oberonc), MIT.
- [rsdoiel/Artemis](https://github.com/rsdoiel/Artemis), BSD-3-Clause.

Every program needed some adaptation, because none of these projects targets
this compiler. Three causes account for nearly all of it:

- Host-specific modules. The KolibriOS samples open and close a `Console`
  module and pause for a keypress on exit. A program here writes to standard
  output and returns.
- Dialect extensions. `BITS`, `ARGNUM`, `ARGS`, `HALT`, and `WCHAR` are not
  Oberon-07. Identifiers holding an underscore are not Oberon-07 either.
- A different `In`. This library's `In.String` reads a *quoted* string, so a
  program reading a bare word or a whole line uses `In.Name` or `In.Line`
  instead. [docs/standard-library.md](../docs/standard-library.md) is the
  reference for what the bundled modules do.

Where a program relied on undefined behaviour that happened to work upstream,
the fix is in the program rather than papered over: `PrimeNumbers` depended on
the JVM zeroing its local arrays, and Oberon-07 leaves a local undefined until
it is assigned.

Two upstream examples are left out for that same reason, because fixing them
would mean rewriting the module under test rather than adapting a program.
Artemis `path-lists` reads an uninitialized pointer -- `PathLists.Decode` tests
its `VAR pathList` parameter before assigning it, and `PathLists.Apply` passes a
local that was never set -- so the program dereferences whatever the stack held
and dies with SIGBUS. Artemis `dstrings-list`, `dictionary`, `utf8-list`,
`ini-parser`, `log`, `dir-checksum`, and the two socket examples need `SYSTEM`
or other host modules this compiler does not provide.
