# Puck: An Oberon for Today

> Make things as simple as possible, but not simpler.

Puck is a working name. Oberon is a moon of Uranus; so is Puck, a smaller and
quicker one. Rename at will.

This document describes a language that forks the Oberon-07 compiler in this
repository. The language keeps Oberon's architecture of thought: modules with
explicit interfaces, a handful of orthogonal types, record extension instead
of classes, garbage collection, runtime checks that trap, and no feature whose
job can be done by one that already exists. It changes three things.

1. **The surface.** Lowercase keywords, braces, newline-terminated statements,
   C-family precedence, escape sequences, and a `for` that covers every loop.
   Odin and Go already did this work on top of Oberon's ideas; Puck follows
   them wherever the result is still recognizably Wirth's.
2. **The sharp edges.** Order-independent declarations at module scope, block
   scoped variables, zero initialization, `return` from anywhere, a real
   `string`, and slices. Each of these removes a rule that existed to make
   a one-pass compiler simple, not to make programs clear.
3. **The outside world.** Sized integers, `f32`, `cstring`, `rawptr`, pointers
   to any type, an address operator, and `foreign` declarations that bind C
   symbols. Procedure values are already plain code addresses and records are
   already laid out as C structs, so most of the interface is a matter of
   admitting what the compiler does.

Everything else stays. There are no methods, interfaces, generics, closures,
exceptions, operator overloading, or macros. Where the document says "no", it
says why.

## A First Look

```puck
module Hello

import Out

proc main() {
    Out.String("Hello, world\n")
}
```

A longer program, showing the shape of the whole language:

```puck
module Shapes

import Out

type Shape* = record {
    name*: string
}

type Circle* = record(Shape) {
    radius*: real
}

type Rect* = record(Shape) {
    width*, height*: real
}

const Pi = 3.14159265358979

proc Area*(s: ^Shape): real {
    switch s {
    case Circle:
        return Pi * s.radius * s.radius
    case Rect:
        return s.width * s.height
    else:
        return 0.0
    }
}

proc Largest(shapes: []^Shape): ^Shape {
    var best: ^Shape
    for s in shapes {
        if best = nil or Area(s) > Area(best) {
            best := s
        }
    }
    return best
}

proc main() {
    var shapes = []^Shape{
        &Circle{name: "unit", radius: 1.0},
        &Rect{name: "page", width: 8.5, height: 11.0},
    }
    for s in shapes {
        Out.String(s.name)
        Out.String(": ")
        Out.Real(Area(s), 10)
        Out.Ln()
    }
    Out.String("largest: ")
    Out.String(Largest(shapes).name)
    Out.Ln()
}
```

An Oberon programmer reads this without a manual. A Go or Odin programmer
reads it without a manual. That is the target.

## What Stays

These are the load-bearing decisions of Oberon, and Puck keeps every one.

* **Modules are the unit of everything.** One module per file, explicit
  imports, explicit exports with the `*` mark, exported variables read-only
  to importers. Initialization runs in dependency order. There is no
  separate interface file; the compiler derives the interface from source.
* **Strong name-based typing.** Two record types with the same fields are
  different types. Variables never convert implicitly between numeric types.
  Conversions are spelled out.
* **Record extension, type tests, and type guards** are the whole story for
  polymorphism. A procedure-typed field is how a record gets behaviour that
  varies by dynamic type.
* **Value and `var` parameters.** Non-`var` parameters are immutable.
* **Garbage collection** through the Boehm collector. There is no manual
  `free`.
* **Runtime checks that terminate.** Index bounds, nil dereference, failed
  guards, narrowing conversions, slice bounds, and `assert` all report a
  source position and exit. Nothing is caught.
* **Integer `div` and `mod` are floored**, as the Report defines them.
  Division by zero traps. Overflow wraps.
* **Sets** are 64-bit vectors with `{0, 3..=5}`, `in`, `+`, `-`, `*`, `/`.
* **Procedure values are code addresses.** Only module-level procedures can
  be values. Nested procedures exist and cannot see the locals of the
  procedure that encloses them, exactly as in Oberon-07.
* **Constant expressions** fold at compile time with exact arithmetic.
* **The compiler is one person's reading.** Hand-written lexer and parser, a
  small typed IR, QBE for the back end.

## Lexical Structure

### Keywords

```text
and      break    case     const    continue div      else
false    for      foreign  if       import   in       is
mod      module   nil      not      or       proc     record
return   switch   true     type     var
```

Predeclared identifiers are not keywords and can be shadowed, as in Oberon:
the type names (`int`, `real`, `bool`, `byte`, `rune`, `string`, `set`,
`cstring`, `rawptr`, `i8` through `u64`, `f32`, `f64`) and the built-in
procedures (`len`, `new`, `copy`, `slice`, `assert`, `abs`, `floor`,
`size_of`, `align_of`).

### Statements end at newlines

A statement ends at a newline when the last token of the line is an
identifier, a literal, `)`, `]`, `}`, `^`, `return`, `break`, or `continue`.
This is Go's rule, and it has Go's two consequences: `else` sits on the same
line as the closing `}` of the branch before it, and the last element of a
multi-line literal ends with a comma. A `;` may separate statements on one
line.

### Comments, identifiers, literals

```puck
// Line comment.
/* Block comment. /* Nests, as Oberon's does. */ */

var snake_case_is_fine = 1          // identifiers may hold underscores
var dec = 1_000_000
var hex = 0xFF
var x = 1.5e-3
var c = 'a'                         // integer constant 97
var nl = '\n'
var s = "tab\there, quote\", backslash\\, nul\0, byte \x41, point \u{1F600}"
var bits = {0, 2..=4}               // set constant
```

Numeric and character literals are untyped constants. A string literal is a
`string`. There is no `0X` character form and no `H` hexadecimal suffix.

### Operators and precedence

Oberon gives `and` and `or` the same precedence as `*` and `+`, so every
relation inside a condition needs parentheses. Puck uses the precedence
every other living language uses.

| Level | Operators |
| --- | --- |
| 1 (tightest) | unary `-` `+` `not` `&` |
| 2 | `*` `/` `div` `mod` `<<` `>>` |
| 3 | `+` `-` |
| 4 | `=` `!=` `<` `<=` `>` `>=` `in` `is` |
| 5 | `and` |
| 6 | `or` |

`=` is equality, `:=` is assignment, as in every Wirth language; declarations
bind with `=`, statements assign with `:=`. `!=` replaces `#`. `and`, `or`, `not` replace `&`, `OR`, `~`. `<<` and `>>` shift integers;
`>>` is arithmetic on signed types and logical on unsigned ones.

```puck
// Oberon
IF (0 <= i) & (i < LEN(a)) & (a[i] # x) THEN ...

// Puck
if 0 <= i and i < len(a) and a[i] != x { ... }
```

## Declarations

Oberon orders a module as `CONST`, `TYPE`, `VAR`, procedures, body, and
requires every name to be declared before its first use. The second rule is
why Oberon-07 cannot express two mutually recursive procedures without a
procedure variable, and why every pointer type needs a separately named
record type declared after it. Both rules are gone.

**Module scope is order independent.** Every declaration in a module sees
every other, in any order. A cycle that does not pass through a pointer or a
procedure type is an error.

```puck
proc IsEven(n: int): bool {
    if n = 0 { return true }
    return IsOdd(n - 1)
}

proc IsOdd(n: int): bool {
    if n = 0 { return false }
    return IsEven(n - 1)
}

type Node = record {
    value: int
    next: ^Node          // no NodeDesc, no forward reference rule
}
```

**Block scope is sequential.** Inside a procedure body, `var`, `const`,
`type`, and nested `proc` declarations may appear anywhere a statement may,
are visible from that point to the end of the enclosing block, and shadow
nothing in the same block.

```puck
proc Sum(xs: []int): int {
    var total = 0
    for x in xs {
        total += x
    }
    return total
}
```

Each kind of declaration has one keyword and one form.

```puck
const Max = 100                      // type from the expression; exact arithmetic
type Point = record { x, y: real }
type Grid = [8][8]byte
type Handler = proc(event: int, data: rawptr)
var count: int                       // zero
var name = "puck"                    // string, from the initializer
var origin: Point                    // all fields zero
proc Dist(a, b: Point): real { ... }
```

A `type` declaration names the type on its right; `type A = B` makes `A`
another name for `B`, as in Oberon. Every `record`, array, slice, pointer,
and `proc` type expression builds a new type.

A module-level variable's initializer must be a constant expression. Anything
that must run goes in `init`.

### Export

The `*` mark exports a module-level constant, type, variable, procedure, or
record field, as in Oberon. An exported variable is read-only outside its
module. An exported record type may have unexported fields.

## Types

| Type | Size | Notes |
| --- | --- | --- |
| `i8` `i16` `i32` `i64` | 1 2 4 8 | two's complement |
| `u8` `u16` `u32` `u64` | 1 2 4 8 | |
| `int` | 8 | another name for `i64`; wide enough for an address |
| `byte` | 1 | another name for `u8` |
| `rune` | 4 | another name for `i32`; a Unicode scalar value |
| `f32` `f64` | 4 8 | IEEE binary32 and binary64 |
| `real` | 8 | another name for `f64` |
| `bool` | 1 | `true`, `false`; matches C `_Bool` |
| `set` | 8 | elements 0 through 63 |
| `string` | 16 | immutable bytes with a length; UTF-8 by convention |
| `[]T` | 16 | slice: a pointer and a length |
| `[n]T` | n × size(T) | array, `n` a constant |
| `^T` | 8 | pointer to any `T`; `nil` |
| `record { ... }` | | fields in order, C layout |
| `proc(...)` | 8 | code address; `nil` |
| `cstring` | 8 | pointer to NUL-terminated bytes; `nil` |
| `rawptr` | 8 | untyped address, C's `void *`; `nil` |

`int`, `real`, `byte`, and `rune` are the names used in ordinary programs.
The sized names exist for record layouts and foreign signatures, and they are
the same types, not distinct ones: `int` and `i64` mix freely because they
are one type. `i32` and `int` do not.

### Constants and conversions

An integer constant is compatible with every integer type that can hold its
value. A real constant is compatible with `f32` and `f64`. A character
literal is an integer constant, so `'a'` assigns to a `byte`, a `rune`, or an
`int`. This generalizes Oberon's rule that an `INTEGER` constant may be
assigned to a `BYTE`, and it is the whole of implicit conversion in the
language. An integer constant does not convert to a real type; write `2.0`.

When a constant has to stand on its own, as in `var x = 1`, an integer
constant is an `int`, a real constant is a `real`, and a character literal is
a `rune`.

Variables convert only through `T(x)`, where `T` is a type name or a
parenthesized type expression.

```puck
var n: int = 300
var b = byte(n)          // traps: 300 is not a byte
var w = i32(n)           // fine
var f = real(n)          // exact
var i = int(2.7)         // 2; truncates toward zero, traps if out of range
var r = rune(b)          // widening is also explicit
var bits = set(0xF0)     // reinterpret; set(x) and u64(s) are free
var p = IntRef(raw)      // rawptr to ^int; the other direction is implicit
```

Narrowing a variable is checked at run time and traps on loss, the way an
index check does. Any pointer type and `cstring` convert implicitly to
`rawptr`; the reverse, and conversions between pointer types or between a
pointer and an `int`, are explicit unchecked reinterpretations that exist for
the foreign interface.

### Strings

`string` is the type Oberon lacks. A value is a pointer and a length, points
at immutable bytes, and is never nil: the zero value is `""`. Literals point
at static data. `+` concatenates and allocates. `=` and `<` compare bytes.
`s[i]` is a `byte`, `s[i:j]` is a view of the same bytes, `len(s)` is the
byte count, and `for i, b in s` visits bytes. Decoding runes is a library
job.

```puck
proc Join(parts: []string, sep: string): string {
    var out = ""
    for i, p in parts {
        if i > 0 { out += sep }
        out += p
    }
    return out
}

var greeting = "hello"
var h = greeting[0]              // 'h' as a byte
var ello = greeting[1:]          // shares storage
var bytes = []byte(greeting)     // copies
var back = string(bytes)         // copies
```

### Arrays and slices

`[n]T` is Oberon's `ARRAY n OF T`: a value, copied on assignment, indexed
with a check. `[]T` is a slice, a pointer and a length, and it is what
Oberon's open array parameter was already becoming: the compiler here passes
open arrays as an address and a length today. Puck lets that pair be a value.

```puck
var a: [4]int                    // four zeros on the stack
var s = a[:]                     // view of all of a
var t = a[1:3]                   // view of a[1], a[2]
var d = new([]int, 100)          // hundred zeros on the heap
var lit = []int{3, 1, 4, 1, 5}   // heap, from a literal
var m = [3]real{1.0, 2.0, 3.0}   // array literal, a value
```

A slice is a view. Assigning a slice copies the view, not the elements;
`copy(dst, src)` copies `min(len(dst), len(src))` elements. Indexing and
slicing are bounds-checked. A nil slice has length zero. Multi-dimensional
arrays are arrays of arrays; a slice of a two-dimensional array is a slice of
rows.

There is no capacity and no `append`. Growing a buffer is written where it is
needed:

```puck
proc Grow(xs: []int, n: int): []int {
    var ys = new([]int, n)
    copy(ys, xs)
    return ys
}
```

If one addition is made after the core settles, it is `append`. The cost is
a third word in every slice and one more built-in; the gain is the single
most used operation in Go. The decision is deferred, not refused.

### Pointers, allocation, and addresses

`^T` points to any `T`, not only to records. `new(T)` returns a zeroed `^T`;
`new([]T, n)` returns a slice of `n` zeroed elements. `p^` dereferences.
Field selection and indexing dereference implicitly, as in Oberon:
`p.next.value` and `rows[i][j]` work through pointers.

`&x` takes the address of a variable, field, element, or composite literal.
A composite literal's address is a fresh heap object, so `&Circle{radius:
1.0}` is how a record is allocated and filled in one expression.

One rule keeps this memory-safe without escape analysis: **a local variable
whose address is taken, or which is sliced, lives on the heap.** The
compiler sees every `&` and every `[:]` in the procedure body, so the
decision is syntactic and made at the declaration. Such a local costs one
allocation per call; all other locals stay on the stack. Parameters are
excluded: the address of a parameter cannot be taken, and an array parameter
cannot be sliced. The caller slices and passes the slice.

Pointers never dangle, exactly as in Oberon, and `&` still gives C the
address of a variable when C needs one.

### Records

```puck
type Entry = record {
    key*: string
    count: int
}

type Timed = record(Entry) {
    when: int
}
```

Fields are declarations, one per line or separated by `;`. A record extends
at most one base and holds the base's fields as a prefix. A record is laid
out as the C struct with the same fields in the same order, with natural
alignment; there is no hidden field inside the record. Heap objects carry
their type descriptor in a header before the object, so a `^T` to a heap
record is also a valid pointer for C.

Assignment copies the whole value when both sides have the same type and the
base prefix when a derived record is assigned to a base, as in Oberon 9.1.

Record literals name their fields; omitted fields are zero. Array and slice
literals are positional.

```puck
var e = Entry{key: "x"}                  // count is 0
var p = &Timed{key: "y", count: 2, when: 0}
```

### Type tests, guards, and switches

Dynamic type exists for pointers to records and for `var` parameters of
record type, as in Oberon. `s is T` tests, `s.(T)` guards and traps, and a
type `switch` guards each arm. When the subject is a pointer, a label that
names a record type `R` means `^R`, so code reads as it did in Oberon.

```puck
if s is Circle {
    Out.Real(s.(Circle).radius, 8)
}

switch s {
case Circle:
    Out.Real(s.radius, 8)            // s is ^Circle in this arm
case Rect:
    Out.Real(s.width, 8)
}
```

A type switch arm retypes the subject for the arm's statements, which is
Oberon-07's type `CASE` under a different keyword.

### Procedures

```puck
proc Clamp(x, lo, hi: int): int {
    if x < lo { return lo }
    if x > hi { return hi }
    return x
}

proc Swap(var a, b: int) {
    var t = a
    a := b
    b := t
}

proc Fill(xs: []int, value: int) {
    for i in 0..<len(xs) {
        xs[i] := value
    }
}
```

Parameter groups are separated by commas, as in Odin. Non-`var` parameters
are immutable: a parameter cannot be assigned, incremented, or have its
address taken. Oberon-07 forbids assignment to structured value parameters
and allows it to scalar ones; Puck makes the rule uniform, which is also
what lets the compiler keep passing structured values by address without a
copy. A slice parameter is a view, so its elements are writable. A `var`
parameter aliases the argument, and a `var` parameter of record type carries
its dynamic type.

`return` is a statement and may appear anywhere. A procedure with a result
type must end every path with one. Nested procedures are declared in the
body like any other local and, as in Oberon-07, see module-level names and
their own locals but not the enclosing procedure's.

Only module-level procedures are values. A `proc` type whose signature uses
only foreign-compatible types has the same representation as a C function
pointer, which the foreign interface relies on.

### Sets

`set` is unchanged from Oberon except for spelling: `{1, 3..=5}` constructs,
`in` tests, and `+ - * /` are union, difference, intersection, and symmetric
difference. `s += {3}` and `s -= {3}` replace `INCL` and `EXCL`.

## Statements

```puck
x := 1                        // assignment
x += 2                        // also -=, for any type with the binary operator
Out.Ln()                      // a call is always written with parentheses
```

`if` has `else if` in place of `ELSIF`:

```puck
if n < 0 {
    sign := -1
} else if n > 0 {
    sign := 1
} else {
    sign := 0
}
```

`for` is the only loop, in three forms:

```puck
for i < n { ... }                  // WHILE
for { ... break ... }              // LOOP, needs break
for i in 0..<n { ... }             // FOR, half-open
for i in 1..=10 { ... }            // FOR, closed
for x in xs { ... }                // elements of an array, slice, or string
for i, x in xs { ... }             // index and element
```

The range forms evaluate their bounds once and declare the loop variables
for the body, where they are read-only. `break` and `continue` apply to the
innermost loop. `REPEAT`, `WHILE ... ELSIF`, and `BY` are gone; the first two
have no users, and a stepping loop is a `for cond` loop with an explicit
increment.

`switch` replaces `CASE`. Labels are constants or closed ranges of an integer
type, or strings, or types. There is no fallthrough; `else` catches the rest,
and a value no label matches, with no `else` present, does nothing.

```puck
switch ch {
case 'a'..='z', 'A'..='Z', '_':
    kind := Ident
case '0'..='9':
    kind := Number
case ' ', '\t', '\n':
    kind := Space
else:
    kind := Other
}
```

`assert(cond)` traps with its position.

## Modules and Programs

```puck
module Strings

import Out
import M = Math                   // qualifier alias, Oberon's IMPORT M := Math
```

A module `X` lives in `X.puck`. Imports are resolved from the directory of
the root source file and then from the bundled library directory, as today.

A module may declare `proc init()`, which runs once before the `init` of any
module that imports it, in the dependency order the driver already computes.
The module given to the compiler must declare `proc main()`, which runs
after every `init`. `init` and `main` take no parameters, return nothing,
and cannot be exported or called.

```puck
module Hailstone

import Out, Program

var cache: []int

proc init() {
    cache := new([]int, 1024)
}

proc main() {
    ...
    Program.Exit(0)
}
```

The `END Name.` trailer, the `BEGIN` body, and the parameterless-call rule
are gone. `Program` keeps the command line and exit status.

## Calling C

The third goal shapes the type system more than the syntax. Every piece
below is small; together they make a C library usable without a shim.

### `foreign` blocks

A `foreign` block declares procedures and variables that the linker
supplies. The optional string names a library; the driver passes it to the
system linker as `-l` followed by the name. Declarations in the block have
no bodies. A trailing `= "symbol"` binds a different link name.

```puck
module Env

import Out

foreign "c" {
    proc getenv(name: cstring): cstring
    proc puts(s: cstring): i32
    proc putString(s: cstring): i32 = "puts"
    var errno: i32
}

proc main() {
    var home = getenv("HOME")
    if home != nil {
        puts(home)
    } else {
        puts("no HOME")
    }
}
```

A foreign procedure is an ordinary procedure to the rest of the module: it
has a type, can be exported, can be a value. A foreign variable is an
ordinary variable at a C address.

### Compatible types

A type is **foreign-compatible** when C has the same representation:

* the sized integers and `int`, `byte`, `rune`, `bool`, `f32`, `f64`, `real`
* `^T` for any `T`, `rawptr`, `cstring`
* `proc` types whose parameters and result are all foreign-compatible and
  have no `var` parameters
* `[n]T` for compatible `T`, as a parameter only, passed by address like a
  C array

Every parameter and the result of a foreign procedure, and the type of a
foreign variable, must be foreign-compatible, with one addition: a `string`
or `[]T` may be a parameter of a foreign procedure, where it becomes two C
parameters, a pointer to the first element and an `int` count. That is the
convention the runtime's C functions use today. `set`, records by value,
`var` parameters, and a `string` or slice anywhere but a foreign parameter
are refused with a diagnostic at the declaration. A record is passed by
`^T`; its layout is already C's.

Hidden arguments are why the restriction is needed and why it is enough.
The compiler's own convention adds a length to each open array and a
descriptor to each record `var` parameter; everything else is already the
platform C convention, because QBE produces it. A signature without those
features has no hidden arguments, so the native call and the foreign call
are the same call.

### Strings across the boundary

`cstring` is a pointer to NUL-terminated bytes. A string literal where a
`cstring` is expected is static data with a terminator, which the compiler
already emits. Otherwise the conversions copy:

```puck
var c = cstring(s)        // GC-allocated copy with a terminator
var s2 = string(c)        // copy up to the terminator; nil gives ""
```

C may keep a `cstring` it was given only while the Puck side also keeps it:
the collector does not see C's heap. The rule is the same for any pointer
handed to C. Memory that C must own outright comes from `malloc`, declared
foreign like anything else.

### Records as C structs

```puck
type Tm = record {
    sec, min, hour, mday, mon, year, wday, yday, isdst: i32
    gmtoff: i64
    zone: cstring
}

foreign "c" {
    proc time(out: ^i64): i64
    proc localtime(t: ^i64): ^Tm
}

proc main() {
    var now = time(nil)
    var t = localtime(&now)          // now lives on the heap: its address is taken
    Out.Int(int(t.year) + 1900, 0)
    Out.Char('-')
    Out.Int(int(t.mon) + 1, 0)
    Out.Ln()
}
```

The field layout matches `struct tm` because both follow the platform's
alignment rules. `size_of(T)` and `align_of(T)` are integer constants for
the cases where C wants a size, and as constants they fit whatever integer
type the C side declares.

### Callbacks

A module-level procedure with a foreign-compatible signature is a C function
pointer.

```puck
foreign "c" {
    proc qsort(base: rawptr, count: u64, size: u64,
               compare: proc(a, b: rawptr): i32)
}

type IntRef = ^int

proc CompareInts(a, b: rawptr): i32 {
    var x = IntRef(a)^
    var y = IntRef(b)^
    if x < y { return -1 }
    if x > y { return 1 }
    return 0
}

proc main() {
    var xs = []int{5, 3, 9, 1}
    qsort(&xs[0], u64(len(xs)), size_of(int), CompareInts)
    for x in xs { Out.Int(x, 4) }
    Out.Ln()
}
```

### C memory as slices

C hands back pointers to arrays with a separate count. `slice(p, n)` views
`n` elements starting at `^T` as a `[]T`, after which every access is
bounds-checked against `n`. The collector ignores pointers outside its
heap, so a slice over `malloc` memory is as safe as its count.

```puck
foreign "c" {
    proc malloc(n: u64): rawptr
    proc free(p: rawptr)
}

type F64Ref = ^f64

proc main() {
    var n = 1024
    var raw = malloc(u64(n) * size_of(f64))
    var buf = slice(F64Ref(raw), n)
    for i in 0..<n { buf[i] := real(i) }
    ...
    free(raw)
}
```

### Variadic C

Only foreign procedures may be variadic; the native language has no
variadic procedures. Arguments in the variadic part must be
foreign-compatible, and an untyped constant there is an `int` or a `real`,
so `%lld` and `%f` are the matching conversions, or convert explicitly.

```puck
foreign "c" {
    proc printf(format: cstring, ...): i32
}

printf("%s is %d\n", cstring(name), i32(age))
```

QBE supports variadic calls directly, so this is a flag on the call.

### What the foreign interface does not do

No C headers are read. No C is generated. Structs are not passed by value.
Unions, bit fields, and `long double` have no spelling; a union is a
`[n]byte` with conversions, which is how C programmers treat them anyway.
Thread-local C variables are not addressed. These limits keep the compiler
from needing a C front end, and every one is worked around with a dozen
lines of C compiled by the host compiler, which the driver already invokes.

## The Standard Library

The bundled modules are rewritten in Puck, and the private `OberonRuntime`
import disappears: `Out`, `In`, `Files`, `Math`, and `Program` declare the
functions of `runtime/standard.c` in `foreign` blocks, the same way a user
module would. The runtime archive is linked into every program, so no
library name is needed.

```puck
module Out

foreign {
    proc oberon_lib_out_string(s: string)       // C sees (const char *, int64_t)
    proc oberon_lib_out_int(value: int, width: int)
    proc oberon_lib_out_ln()
}

proc String*(s: string) { oberon_lib_out_string(s) }

proc Int*(value, width: int) { oberon_lib_out_int(value, width) }
proc Ln*() { oberon_lib_out_ln() }
```

`Strings` shrinks to what `string` does not already do. `Math` can declare
libm directly. The signatures of `Files` and `In` change from `ARRAY OF CHAR`
to `string` and `[]byte`. A `Utf8` module decodes runes.

## Left Out, and Why

* **Methods and interfaces.** Go's receivers and interfaces are its biggest
  departure from Oberon, and they bring dispatch tables and structural
  typing. Record extension plus procedure-valued fields express the same
  programs with explicit dispatch. Wirth never adopted Oberon-2's type-bound
  procedures; Puck does not add them.
* **Generics.** A generic container is written once per element type, as in
  Oberon and as in Go for a decade. Slices and `copy` cover the common case.
* **Closures.** A procedure value is one word and C can call it. Capturing
  would cost both properties.
* **Multiple return values.** `var` parameters return several results, and
  status codes are the error convention. One mechanism is enough.
* **Exceptions.** Language failures trap. Library failures return status.
  The Report's position is kept.
* **Enumerations.** Constants and `switch` do the work, and C enums are
  integers. A `set` of an enumeration would be the one strong argument, and
  it is not strong enough yet.
* **Bare composite literals in control headers.** `if p = (Point{...}) {`
  needs the parentheses, as in Go, because the `{` would otherwise open the
  block. The alternative is a different bracket for sets or for literals,
  and both are worse.
* **Operator overloading, macros, conditional compilation, inline C.** None
  of these are Oberon, and each has a cheaper substitute.

## Grammar

```ebnf
module      = "module" ident { declaration } .
declaration = import | const | type | var | proc | foreign .

import      = "import" importitem { "," importitem } .
importitem  = ident [ "=" ident ] .
const       = "const" identdef "=" expr .
type        = "type" identdef "=" typeexpr .
var         = "var" identlist ( ":" typeexpr [ "=" expr ] | "=" expr ) .
proc        = "proc" identdef signature block .
signature   = "(" [ params ] ")" [ ":" typeexpr ] .
params      = param { "," param } .
param       = [ "var" ] identlist ":" typeexpr .

foreign     = "foreign" [ string ] "{" { foreigndecl } "}" .
foreigndecl = "proc" identdef "(" [ params [ "," "..." ] | "..." ] ")"
                     [ ":" typeexpr ] [ "=" string ]
            | "var" identdef ":" typeexpr [ "=" string ] .

identdef    = ident [ "*" ] .
identlist   = identdef { "," identdef } .
qualident   = [ ident "." ] ident .

typeexpr    = qualident
            | "[" expr "]" typeexpr
            | "[" "]" typeexpr
            | "^" typeexpr
            | "record" [ "(" qualident ")" ] "{" { field } "}"
            | "proc" signature .
field       = identlist ":" typeexpr .

block       = "{" { statement } "}" .
statement   = [ assignment | call | if | for | switch
              | "return" [ expr ] | "break" | "continue"
              | const | type | var | proc ] .
assignment  = designator ( ":=" | "+=" | "-=" ) expr .
call        = designator .                       (* must end in a call selector *)
if          = "if" expr block { "else" "if" expr block } [ "else" block ] .
for         = "for" [ expr | ident [ "," ident ] "in" iterable ] block .
iterable    = expr [ ( "..<" | "..=" ) expr ] .
switch      = "switch" expr "{" { "case" labels ":" { statement } }
                               [ "else" ":" { statement } ] "}" .
labels      = label { "," label } .
label       = expr [ "..=" expr ] .              (* a qualident may name a type *)

expr        = andexpr { "or" andexpr } .
andexpr     = relexpr { "and" relexpr } .
relexpr     = addexpr [ relation addexpr | "is" qualident ] .
relation    = "=" | "!=" | "<" | "<=" | ">" | ">=" | "in" .
addexpr     = mulexpr { ( "+" | "-" ) mulexpr } .
mulexpr     = unary { ( "*" | "/" | "div" | "mod" | "<<" | ">>" ) unary } .
unary       = ( "-" | "+" | "not" | "&" ) unary | primary .
primary     = number | character | string | "true" | "false" | "nil"
            | setlit | composite | designator | "(" expr ")" .
setlit      = "{" [ element { "," element } ] "}" .
element     = expr [ "..=" expr ] .
composite   = typeexpr "{" [ elements ] "}" .
elements    = ( expr | ident ":" expr ) { "," ( expr | ident ":" expr ) } [ "," ] .
designator  = qualident { selector } .
selector    = "." ident
            | "[" expr "]"
            | "[" [ expr ] ":" [ expr ] "]"
            | "^"
            | "." "(" qualident ")"
            | "(" [ expr { "," expr } ] ")" .    (* call, conversion, or built-in *)
```

Newline termination is lexical, as described above, and does not appear in
the grammar. As in the present parser, `Out.Int` and `T(x)` are parsed as
designators and resolved by semantic analysis.

## What Changes in the Compiler

The pipeline, the IR's shape, QBE, the collector, the driver's module graph,
and the runtime-check convention all stay. The work is in each stage's
vocabulary, not its structure.

**Lexer.** New keywords and operators, the newline rule, escape sequences,
`0x` and `_` in numbers, character literals as integer constants. The lexer
tracks only the last significant token, as Go's does.

**Parser.** Braces and the block grammar. Declarations as statements.
Composite literals, with the control-header restriction carried as one flag
down the expression parser. `foreign` blocks.

**Semantic analysis.** Two passes over module scope: declare every name,
then resolve, with cycle detection through non-pointer types. Block scopes
on a stack. `Type` gains `Int { width, signed }`, `Float { width }`,
`Slice`, `String`, `CString`, `RawPtr`, and `Pointer` to any type; the
record-only check at `PointerBase` goes. The constant-range compatibility
rule replaces the `ByteRange` case of `assign_kind`. Parameter immutability
is a flag on the symbol. Address-taken and sliced locals are found by a scan
of the body before slots are assigned and are lowered to a heap object plus
a pointer slot. Foreign procedures are symbols with no body and a
compatibility check on their signature.

**IR.** `Ty` gains the sized integers and `F32`; `Bool` becomes one byte.
A string or slice value is sixteen bytes of storage; as a parameter it is
two scalar arguments, which open arrays already are and which is also what
a foreign procedure receives, and as a result it is returned through a
hidden out-pointer. `Call` gains a `variadic`
flag. `Global` gains `extern`. A module records the libraries its `foreign`
blocks named.

**QBE lowering.** Sub-word loads and stores (`loadsb`, `loadub`, `storeh`,
and so on), `s` for `f32`, the `...` marker in variadic calls, and `export`
versus external symbols.

**Runtime.** String concatenation, comparison, and the `string`, `cstring`,
and `[]byte` conversions; slice allocation; nothing else. The collector and
the check-failure reporting are unchanged.

**Driver.** `.puck` files, `-l` flags from `foreign` strings, `main` and
`init` in place of module bodies, and the removal of the bundled-only
private import.

**Library and tests.** The bundled modules are rewritten in Puck. The corpus
is translated program by program; each translation is a test of the
language design as much as of the compiler.

### A sequence

1. Surface syntax, newline rule, precedence, block scope, zero
   initialization, `return`, `break`, `continue`, `for`, `switch`, and
   `init`/`main`. The type system is still Oberon's. The corpus is
   translated and passes.
2. Order-independent module scope.
3. Sized integers, `f32`, one-byte `bool`, `^T` to any type, constant
   compatibility, `T(x)` conversions, `&`, and the heap rule for
   address-taken locals.
4. Slices and `string`, with `copy`, `slice`, `len`, and the conversions.
5. `foreign`, `cstring`, `rawptr`, variadic calls, link libraries. The
   bundled modules move onto `foreign` and the private interface is removed.

Each step leaves a working compiler and a passing corpus. Step 1 is the
largest and the least risky; step 5 is the smallest and the reason for the
rest.
