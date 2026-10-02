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
   Go's precedence, escape sequences, and a `for` that covers every loop.
   Odin and Go already did this work on top of Oberon's ideas; Puck follows
   them wherever the result is still recognizably Wirth's.
2. **The sharp edges.** Order-independent declarations at module scope, block
   scoped variables, zero initialization, `return` from anywhere, a real
   `string`, and slices. Each of these removes a rule that existed to make
   a one-pass compiler simple, not to make programs clear.
3. **The outside world.** Sized integers, `f32`, `cstring`, `rawptr`, pointers
   to any type, an address operator, integer bit operators, and `foreign`
   declarations that bind C symbols. Procedure values are already plain code
   addresses and records are already laid out as C structs, so most of the
   interface is a matter of admitting what the compiler does.

Everything else stays. There are no methods, interfaces, generics, closures,
exceptions, operator overloading, or macros. Where the document says "no", it
says why.

**Puck is a separate repository that begins as a copy of this one.** The
Oberon-07 compiler here keeps its goal, its corpus, and its instructions
unchanged; nothing in it moves toward Puck, and an Oberon conformance test
that Puck translates stays behind as an Oberon conformance test. After the
copy the two share no code. A fix found in one is ported to the other by
hand when it is wanted, which should be seldom: the parts most likely to need
fixes, the collector and QBE, are vendored upstream code in both.

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

An Oberon programmer reads this with a glance at the keyword table. A Go or
Odin programmer reads it with a glance at the table below, which lists the
places where Puck keeps Wirth's spelling against theirs. That is the target:
no manual, one table.

| Puck | Go | Odin | Why Puck keeps it |
| --- | --- | --- | --- |
| `x := e` assigns, `var x = e` declares | `x = e`, `x := e` | `x = e`, `x := e` | Wirth: `=` is equality everywhere |
| `and` `or` `not` | `&&` `\|\|` `!` | `&&` `\|\|` `!` | words for logic, symbols for bits |
| `p^` dereferences | `*p` | `p^` | Oberon's spelling, and Odin's |
| `proc F() {` | `func F() {` | `F :: proc() {` | one keyword, one form |
| `^T` points to `T` | `*T` | `^T` | pointer and dereference share a mark |
| `name*` exports | `Name` | `@export` | Oberon's mark; case is the programmer's |

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
  `free` for Puck's own objects.
* **Runtime checks that terminate.** Index bounds, nil dereference, failed
  guards, narrowing conversions, slice bounds, shift counts, and `assert`
  all report a source position and exit. Nothing is caught.
* **Integer `div` and `mod` are floored**, as the Report defines them.
  Overflow is defined by operation: `+`, `-`, `*`, and unary `-` wrap in
  two's complement at the operand's width; `div` and `mod` trap on a zero
  divisor and on the one quotient that does not fit, the minimum value
  divided by `-1`; `abs` traps on the minimum value. These are the present
  compiler's rules except the `-1` case, which is one more check.
* **Sets** are 64-bit vectors with `{0, 3..=5}`, `in`, `+`, `-`, `*`, `/`.
* **Procedure values are code addresses.** Only module-level procedures can
  be values. Nested procedures exist and cannot see the locals of the
  procedure that encloses them, exactly as in Oberon-07.
* **Constant expressions** fold at compile time in checked arithmetic. An
  integer constant may hold any value from −2⁶³ through 2⁶⁴ − 1, so every
  value of every integer type can be written; an intermediate outside that
  range is a diagnostic, not a wrap. Real constants are `f64`, rounded at
  every operation. The present compiler folds in checked 64-bit arithmetic;
  the wider range is the one change.
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
line as the closing `}` of the branch before it, and a multi-line bracketed
list ends each line with a comma. A trailing comma is therefore permitted
before the closing bracket of a call, a parameter list, a set literal, and a
composite literal. A `;` may separate statements on one line.

```puck
Out.Int(
    n,
    0,
)
```

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
relation inside a condition needs parentheses. Puck uses Go's levels, which
also put the bit operators where C programmers reach for them: `&` and the
shifts with `*`, `|` and `~` with `+`.

| Level | Operators |
| --- | --- |
| 1 (tightest) | unary `-` `+` `not` `~` `&` |
| 2 | `*` `/` `div` `mod` `<<` `>>` `&` |
| 3 | `+` `-` `\|` `~` |
| 4 | `=` `!=` `<` `<=` `>` `>=` `in` `is` |
| 5 | `and` |
| 6 | `or` |

`=` is equality, `:=` is assignment, as in every Wirth language; declarations
bind with `=`, statements assign with `:=`. `!=` replaces `#`. `and`, `or`,
`not` replace `&`, `OR`, `~` and apply to `bool`. Binary `&`, `|`, `~` are
and, or, and exclusive or on two integers of the same type; unary `~` is the
complement; `<<` and `>>` shift an integer by an integer count, and `>>` is
arithmetic on signed types and logical on unsigned ones. Unary `&` is the
address operator. Integer `/` is an error, as in Oberon: write `div`.

Two parses deserve a warning, because they differ from C. `not x in s` is
`(not x) in s`, a type error; write `not (x in s)`. `1 << 2 + 1` is
`(1 << 2) + 1`, which is 5, where C gives 8.

```puck
// Oberon
IF (0 <= i) & (i < LEN(a)) & (a[i] # x) THEN ...

// Puck
if 0 <= i and i < len(a) and a[i] != x { ... }
```

## Declarations

Oberon orders a module as `CONST`, `TYPE`, `VAR`, procedures, body, and
requires every name to be declared before its first use. Oberon-07 softens
the second rule once, for a pointer to a record declared later in the same
scope, and nowhere else: two mutually recursive procedures need a procedure
variable, and a constant or type cannot refer to one declared below it. Both
rules are gone.

**Module scope is order independent.** Every declaration in a module sees
every other, in any order. A type may not contain itself inline: a cycle
through record fields and array elements is an error, while a pointer, a
slice, a `string`, or a `proc` type is an indirection and breaks the cycle,
because its size does not depend on what it refers to. A cycle among
constants, or among type names that are mere aliases, is an error.

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
    next: ^Node
}

type Tree = record {
    label: string
    children: []Tree             // legal: a slice is an indirection
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
const Max = 100                      // type from the expression
type Point = record { x, y: real }
type Grid = [8][8]byte
type Handler = proc(event: int, data: rawptr)
var count: int                       // zero
var name = "puck"                    // string, from the initializer
var origin: Point                    // all fields zero
proc Dist(a, b: Point): real { ... }
```

A `type` declaration names the type on its right; `type A = B` makes `A`
another name for `B`, as in Oberon. Every `record` type expression builds a
new type. Two array, slice, pointer, or `proc` type expressions denote the
same type when their components do: `[]int` written in a parameter and
`[]int` written in a literal are one type, `^Node` in a field is `^Node` in
a result, and two `proc` types with the same parameter types, modes, and
result are one type whatever the parameter names. Oberon makes each named
pointer and procedure type distinct and then adds compatibility rules to let
them meet; Puck makes them identical by structure and needs no such rules.

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
| `rune` | 4 | another name for `i32`; conventionally a Unicode scalar value |
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
are one type. `i32` and `int` do not. `rune` is an integer that holds a
Unicode scalar value by convention; the type does not enforce the range, the
`Utf8` module does, so `var r: rune = -1` is legal and meaningless, as it is
in Go.

Arithmetic on an n-bit integer type wraps modulo 2ⁿ. Both operands of a
binary arithmetic or bit operator have the same type; a constant adapts to
the other operand. The count of a shift may be any integer type and must lie
in `0..<width` of the shifted operand, checked at compile time for a constant
and at run time otherwise; the result has the shifted operand's type.

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

Variables convert only through `T(x)`, where `T` is a type name or a type
expression beginning with `[` or `^`. Each conversion is one of four kinds:
exact, rounding, checked, or reinterpreting. A checked conversion traps
when the value has no representative in the target; a rounding one never
traps; a reinterpreting one changes the type and not the bits. A conversion
of a constant is evaluated at compile time, and a diagnostic replaces the
trap.

| From | To | Kind | Rule |
| --- | --- | --- | --- |
| integer | integer | checked | the value must fit; narrowing and signedness changes are value checks, so `u64(-1)` traps |
| integer | `f32` `f64` | rounding | to nearest; `real(n)` is exact below 2⁵³ |
| `f32` `f64` | integer | checked | truncates toward zero, then the integer must fit; NaN and infinity trap |
| `f64` | `f32` | rounding | to nearest; overflow gives infinity |
| `f32` | `f64` | exact | |
| `i64` `u64` | `set` | reinterpreting | narrower integers are converted to `int` or `u64` first |
| `set` | `i64` `u64` | reinterpreting | |
| pointer, `cstring`, `proc` | `rawptr` | exact, implicit | |
| `rawptr` | pointer, `cstring`, `proc` | reinterpreting, unchecked | for the foreign interface |
| pointer | pointer | reinterpreting, unchecked | for the foreign interface |
| pointer, `rawptr` | `int` `u64` | reinterpreting, unchecked | for the foreign interface; and back |
| `string` | `[]byte` `cstring` | copy | see below |
| `[]byte` `cstring` | `string` | copy | see below |

```puck
var n: int = 300
var b = byte(n)          // traps: 300 is not a byte
var w = i32(n)           // fine
var f = real(n)          // exact for this n
var i = int(2.7)         // 2; truncates toward zero
var r = rune(b)          // widening is also explicit
var bits = set(0xF0)     // reinterpret; set(x) and u64(s) are free
var p = IntRef(raw)      // rawptr to ^int; the other direction is implicit
```

The unchecked conversions exist for the foreign interface and are
**programmer-checked operations**: the compiler trusts the programmer about
what the address holds, as it trusts a `foreign` signature and the count
given to `slice`. Everything else in the language is checked by the compiler
or the runtime.

### Strings

`string` is the type Oberon lacks. A value is a pointer and a length, points
at immutable bytes, and is never nil: the zero value is `""`. Literals point
at static data. `+` concatenates and allocates. `=` and `<` compare bytes as
unsigned values, lexicographically, with a shorter prefix ordering first and
a NUL byte comparing like any other. `s[i]` is a `byte` and cannot be
assigned or addressed, `s[i:j]` is a view of the same bytes, `len(s)` is the
byte count, and `for i, b in s` visits bytes. Decoding runes is a library
job.

Concatenation in a loop copies the accumulated prefix each time, so joining
*n* pieces with `+=` copies a quadratic number of bytes. The linear form
measures, allocates once, and converts:

```puck
proc Join(parts: []string, sep: string): string {
    var n = 0
    for i, p in parts {
        if i > 0 { n += len(sep) }
        n += len(p)
    }
    var buf = new([]byte, n)
    var at = 0
    for i, p in parts {
        if i > 0 { at += copy(buf[at:], sep) }
        at += copy(buf[at:], p)
    }
    return string(buf)
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
var a: [4]int                    // four zeros
var s = a[:]                     // view of all of a
var t = a[1:3]                   // view of a[1], a[2]
var d = new([]int, 100)          // hundred zeros on the heap
var lit = []int{3, 1, 4, 1, 5}   // heap, from a literal
var m = [3]real{1.0, 2.0, 3.0}   // array literal, a value
```

A slice is a view. Assigning a slice copies the view, not the elements.
`copy(dst, src)` copies `min(len(dst), len(src))` elements from a `[]T` to a
`[]T` of the same element type, or from a `string` to a `[]byte`, behaves as
`memmove` when the views overlap, and returns the count copied. Indexing and
slicing are bounds-checked; `new([]T, n)` and `slice(p, n)` trap on a
negative `n` or a byte size that overflows. A nil slice has length zero.
Multi-dimensional arrays are arrays of arrays; a slice of a two-dimensional
array is a slice of rows.

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
`p.next.value` and `rows[i][j]` work through pointers. A `^D` is assignable
to a `^B` when record `D` extends record `B`, as in Oberon; no other pointer
conversion is implicit except to `rawptr`.

`&x` takes the address of a variable, field, element, or composite literal.
A composite literal's address is a fresh heap object, so `&Circle{radius:
1.0}` is how a record is allocated and filled in one expression.

**Roots.** Every designator has a root. Reading from the left, the root is
the variable the designator starts from, until the path passes through a
pointer dereference, written or implicit, or indexes a slice or a string;
from there the root is the storage that indirection reaches, which is on the
heap, in static data, or belongs to C. The rules for `&` and for slicing are
stated in terms of roots.

**The heap rule.** A local variable that is the root of an `&` operand or of
a slice expression `[i:j]` lives on the heap. The compiler sees every `&`
and every `[:]` in the procedure body, so the decision is syntactic and made
at the declaration; such a local costs one allocation each time its
declaration executes, and all other locals stay on the stack. A promoted
local declared inside a loop is a fresh object on every iteration, so the
addresses taken in different iterations are distinct and each object lives
as long as something points at it. `return r.buf[:]` promotes `r`, the
root, not the field. A module-level variable is static storage and needs no
promotion.

**Parameters are not roots of addresses.** A parameter, `var` or not, may
not be the root of an `&` operand or a slice expression. Both kinds alias
the caller's storage, which may be a stack local in a procedure that took no
addresses, and the callee cannot know. So `&r.field`, `&a[i]`, and
`r.buf[:]` are refused when `r` and `a` are parameters; the caller addresses
or slices and passes the result. A pointer or slice parameter is itself an
indirection: `&p.field`, `&xs[i]`, and `xs[a:b]` are fine, because their
roots are the referents, not the parameters.

**Addresses are of writable storage.** `&` requires a designator that could
stand on the left of `:=`. Immutability belongs to a root and is inherited
through fields and elements of inline storage: a non-`var` parameter, an
exported variable seen from another module, a `for` loop variable, a
constant, and the bytes of a `string` are all read-only, and so is anything
selected or indexed inside them without passing through an indirection. The
referent of a pointer held in read-only storage is writable, as it is in
Oberon, where a value parameter of pointer type still lets `p.f := x`.

Within these rules, pointers never dangle, exactly as in Oberon, and `&`
still gives C the address of a variable when C needs one. Pointers that come
from C or from an unchecked conversion, and slices built by `slice`, are
outside them; see "Calling C".

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
at most one base and holds the base's fields as a prefix. A record with no
base is laid out as the C struct with the same fields in the same order,
under the platform's alignment rules; a record with a base is laid out as
the C struct whose first member is the complete base struct, followed by
the new fields. This is what the compiler does today, and it differs from a
flattened struct only when the base ends in padding: `record { x: int; y:
byte }` extended with `z: byte` puts `z` at offset 16, where a flat C struct
would put it at 9. There is no hidden field inside a record. Heap objects
carry their type descriptor in a header before the object, so a `^T` to a
heap record is also a valid pointer for C.

Assignment copies the whole value when both sides have the same type and the
base prefix when a derived record is assigned to a base, as in Oberon 9.1.

Record literals name their fields; omitted fields are zero. Array and slice
literals are positional.

```puck
var e = Entry{key: "x"}                  // count is 0
var p = &Timed{key: "y", count: 2, when: 0}
```

### Dynamic type

A record on the heap carries its descriptor in a header before the object; a
record held inline in a variable, field, or element has no header, and its
dynamic type is its static type. Oberon keeps the two apart by letting
pointers come only from `NEW`. Puck has `&`, so it needs a rule:

**The address of a record is taken only from a whole variable.** `&v` for a
local or module-level variable of record type is permitted; `&r.inner` and
`&a[i]` are refused when the field or element is itself a record. An
address-taken local of record type lives on the heap, where `new` gives it
a header; an address-taken module-level variable of record type is emitted
in static data behind the same header. Every `^R` to a record that Puck code
produces therefore points at a header or is `nil`, and `is`, `.(T)`, and a
type `switch` read the header as the compiler does today.

An array of records that C must see whole is passed as `&a`, a `^[n]R`,
which converts to `rawptr`. A record that must be addressed on its own is
allocated on its own, which is how Oberon programs are already written: an
array of pointers, not an array of records.

A pointer that enters through a `foreign` result or an unchecked conversion
carries whatever lies before it. A type test on such a pointer is a
programmer-checked operation, like the conversion or the call that produced
it; a pointer from C is used at its declared type.

`var` parameters of record type carry their dynamic type as a hidden
argument, as in Oberon. The caller supplies it from the static type when the
actual is a variable, field, or element, from the header when the actual is
`p^`, and from its own incoming hidden argument when the actual is itself a
record `var` parameter, so a derived record keeps its dynamic type through
any chain of forwarding calls. The compiler does this today.

### Type tests, guards, and switches

Dynamic type exists for pointers to records and for `var` parameters of
record type. `s is T` tests, `s.(T)` guards and traps, and a type `switch`
guards each arm. When the subject is a pointer, a label that names a record
type `R` means `^R`, so code reads as it did in Oberon.

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

The subject of a type `switch` is the name of a variable or parameter, as
the case variable is in Oberon-07; a type test on any other expression is
written with `is` and a guard. Each arm of a type `switch` names exactly one
type and retypes the subject for the arm's statements, which is Oberon-07's
type `CASE` under a different keyword. Arms are tested in source order and
the first match is taken, so an arm for a base type hides a later arm for
one of its extensions; the compiler reports the hidden arm.

A pointer subject is read once, when the `switch` is entered, and within an
arm its name denotes that copy, read-only, at the arm's type. A procedure
called from the arm or an alias such as `alias^ := new(Shape)` may change
the original variable, and cannot change what the arm is looking at; the
fields of the referent stay writable, since the copy is a pointer held in
read-only storage. A `var` parameter of record type names the caller's
storage directly, and assignment to a record cannot change its dynamic type,
so there the name denotes the parameter itself.

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

A call whose result is not used may stand as a statement. Oberon forbids
this so that no result is dropped by accident; nearly every C function
returns a status, and a discard mark on every `printf` would cost more than
the mistakes the rule prevents.

Only module-level procedures are values. A `proc` type whose signature uses
only foreign-compatible types has the same representation as a C function
pointer, which the foreign interface relies on.

### Sets

`set` is unchanged from Oberon except for spelling: `{1, 3..=5}` constructs,
`in` tests, and `+ - * /` are union, difference, intersection, and symmetric
difference. `s += {3}` and `s -= {3}` replace `INCL` and `EXCL`. A `set` is
for sets; bit manipulation of integers uses the integer operators.

## Statements

```puck
x := 1                        // assignment
x += 2                        // also -=, for any type with the binary operator
Out.Ln()                      // a call is always written with parentheses
puts(s)                       // a function call as a statement discards the result
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
innermost loop. `REPEAT`, `WHILE ... ELSIF`, and `BY` are gone. Each has
users in the corpus, and each is a `for` with a condition and an explicit
increment or a `break`; the translation is mechanical and the loop vocabulary
drops from four forms to one.

`switch` replaces `CASE`. Labels are constants or closed ranges of an integer
type, or strings, or types. There is no fallthrough; `else` catches the rest.
A value no label matches, with no `else` present, does nothing. This is a
change: Oberon-07 has no `ELSE`, so the present compiler traps there. Puck
has `else`, and a programmer who wants the trap writes `else: assert(false)`.

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
below is small; together they make a C library usable without a shim for the
common case, and with a few lines of C for the rest.

### Platforms

Puck assumes an LP64 C platform on x86-64 or AArch64 under macOS or Linux
with glibc, which is where QBE and this compiler run today. `int` is C's
`long`, `intptr_t`, and `ssize_t`; `u64` is `size_t`. Record layouts match
C because both follow the platform's alignment rules, and the examples below
are checked against those platforms, not against C in general. Puck has no
conditional compilation. A binding that differs by platform is written in C,
where the preprocessor already exists, and compiled alongside the module.

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
ordinary variable at a C data symbol. C's `errno` is not one: it is a
thread-local reached through a function whose name differs by platform, so
it is bound through a companion C file, below. The same goes for anything C
defines as a macro.

### Companion C

The driver accepts `.c` files on the command line beside the root module,
compiles each with the host `cc` it already invokes for linking, and links
the objects into the program. A platform-dependent symbol, a macro, a
thread-local, or a struct passed by value gets a C function with a plain
signature, declared `foreign` like anything else:

```c
/* env_shim.c */
#include <errno.h>
int puck_errno(void) { return errno; }
```

```puck
foreign {
    proc puck_errno(): i32
}
```

This is the whole of Puck's answer to platform variation.

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

These limits are the initial implementation boundary, not facts about the
ABI. The compiler's own convention adds a length to each open array and a
descriptor to each record `var` parameter, and returns a string or slice
through a hidden out-pointer; a signature without those features is the
platform C convention, because QBE produces it. QBE can also pass and return
C aggregates by value, which is how structs by value and two-word results
would be admitted later if they earn their place. Narrow parameters and
results use QBE's sub-word ABI types so that an `i8` or a `bool` crosses the
boundary as C expects.

### Strings across the boundary

`cstring` is a pointer to NUL-terminated bytes. A string literal where a
`cstring` is expected is static data with a terminator, which the compiler
already emits. Otherwise the conversions copy:

```puck
var c = cstring(s)        // GC-allocated copy with a terminator
var s2 = string(c)        // copy up to the terminator; nil gives ""
```

A `string` may contain NUL bytes and a `cstring` cannot express them:
`cstring(s)` appends a terminator, and C sees the prefix before the first
NUL that `s` already held. A `string` passed directly to a foreign
parameter arrives as pointer and length, and C sees all of it.

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

The field layout matches `struct tm` on the named platforms. `size_of(T)`
and `align_of(T)` are integer constants for the cases where C wants a size,
and as constants they fit whatever integer type the C side declares.

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
`n` elements starting at `^T` as a `[]T`, after which every index is checked
against `n`. The count comes from the programmer, not from the pointer: a
`^T` carries no extent, so `slice(new(int), 1000)` is accepted by the
compiler and overruns a managed object while every index check passes.
`slice` is therefore a programmer-checked operation wherever its pointer
came from, and a view of managed storage is written with `[:]`, which the
compiler can check. The collector ignores pointers outside its heap, so a
slice over C's memory does not keep that memory alive and does not try to.

```puck
foreign "c" {
    proc malloc(n: u64): rawptr
    proc free(p: rawptr)
}

type F64Ref = ^f64

proc main() {
    var n = 1024
    var raw = malloc(u64(n) * size_of(f64))
    assert(raw != nil)
    var buf = slice(F64Ref(raw), n)
    for i in 0..<n { buf[i] := real(i) }
    ...
    free(raw)
}
```

`slice`, the unchecked pointer conversions, and type tests on foreign
pointers are **programmer-checked operations**. The programmer owes the
compiler four things it cannot verify: that the allocation succeeded, that
the address is aligned for `T`, that `n` elements are really there, and that
the memory outlives every view of it. A bounds check cannot catch a use
after `free`. The guarantee that pointers never dangle covers the managed
subset of the language; `foreign`, `slice`, and the unchecked conversions
are where it ends, and each is a name that can be searched for.

### Variadic C

Only foreign procedures may be variadic; the native language has no
variadic procedures. Arguments in the variadic part must be
foreign-compatible scalars or pointers and undergo C's default argument
promotions: `f32` becomes `f64`, and `bool`, `i8`, `u8`, `i16`, and `u16`
become `i32`. An untyped constant there is an `int` or a `real`, so `%lld`
and `%f` are the matching conversions, or convert explicitly. A `string` or
slice is not accepted in the variadic part; pass `cstring(s)`.

```puck
foreign "c" {
    proc printf(format: cstring, ...): i32
}

printf("%s is %d\n", cstring(name), i32(age))
```

QBE supports variadic calls directly, given the position at which the
variadic arguments begin; the IR records that position after a `string` or
slice parameter has been expanded into two.

### What the foreign interface does not do

No C headers are read. No C is generated. Structs are not passed by value.
Unions, bit fields, and `long double` have no spelling; a union is a
`[n]byte` with conversions, which is how C programmers treat them anyway.
Thread-local C variables and macros are not addressed. These limits keep the
compiler from needing a C front end, and every one is worked around with a
dozen lines of companion C.

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

Rebinding is not enough on its own. The C functions were written for
`ARRAY OF CHAR`, which is NUL-terminated, and several stop at the first NUL
inside the length they are given; `oberon_lib_out_string` does. A Puck
`string` is length-delimited, so each buffer-taking function in
`standard.c` is audited for terminator assumptions as it is rebound, and
`Out.String("a\0b")` writes three bytes.

`Strings` shrinks to what `string` does not already do. `Math` can declare
libm directly. The signatures of `Files` and `In` change from `ARRAY OF CHAR`
to `string` and `[]byte`. A `Utf8` module decodes runes:
`Utf8.Decode(s: string, i: int, var r: rune): int` returns the byte length
of the scalar beginning at `i`, delivering U+FFFD and a length of one for a
malformed sequence, so a loop over a string always advances; `Utf8.Encode(r:
rune, buf: []byte): int` writes a scalar and returns its length.

The library offers no hash table, no generic sort, and no formatting into
strings at first. These are costs the design accepts, not oversights, and
they fall on applications as they do in Oberon: a string-keyed map is a page
written per value type, with `Strings.Hash` doing the arithmetic; a sort is
written per element type with its comparison inline, since a `proc` value
carries no context. If a second program needs the same page, the library
grows by that page.

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
  of these are Oberon, and each has a cheaper substitute; for the last two
  the substitute is a companion C file.

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
params      = param { "," param } [ "," ] .
param       = [ "var" ] identlist ":" typeexpr .

foreign     = "foreign" [ string ] "{" { foreigndecl } "}" .
foreigndecl = "proc" identdef "(" [ params [ "..." ] | "..." ] ")"
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
addexpr     = mulexpr { ( "+" | "-" | "|" | "~" ) mulexpr } .
mulexpr     = unary { ( "*" | "/" | "div" | "mod" | "<<" | ">>" | "&" ) unary } .
unary       = ( "-" | "+" | "not" | "~" | "&" ) unary | primary .
primary     = number | character | string | "true" | "false" | "nil"
            | setlit | composite | conversion | designator | "(" expr ")" .
setlit      = "{" [ element { "," element } [ "," ] ] "}" .
element     = expr [ "..=" expr ] .
composite   = typeexpr "{" [ elements ] "}" .
elements    = ( expr | ident ":" expr ) { "," ( expr | ident ":" expr ) } [ "," ] .
conversion  = ( "[" [ expr ] "]" typeexpr | "^" typeexpr ) "(" expr ")" .
designator  = qualident { selector } .
selector    = "." ident
            | "[" expr "]"
            | "[" [ expr ] ":" [ expr ] "]"
            | "^"
            | "." "(" qualident ")"
            | "(" [ argument { "," argument } [ "," ] ] ")" .
argument    = expr | typeexpr .                  (* a type only where a built-in takes one *)
```

Newline termination is lexical, as described above, and does not appear in
the grammar. As in the present parser, `Out.Int` and `T(x)` are parsed as
designators and resolved by semantic analysis; a conversion whose type is
spelled with `[` or `^` has its own production, since neither token can
begin an expression. An `argument` that is a type expression beginning with
`[` or `^` is likewise unambiguous; one that is a `qualident` is resolved by
semantic analysis, which knows whether the callee is `new`, `size_of`, or
`align_of` and expects a type.

## What Changes in the Compiler

The pipeline, the IR's shape, QBE, the collector, the driver's module graph,
and the runtime-check convention all stay. The work is in each stage's
vocabulary, not its structure.

**Lexer.** New keywords and operators, the newline rule, escape sequences,
`0x` and `_` in numbers, character literals as integer constants. The lexer
tracks only the last significant token, as Go's does.

**Parser.** Braces and the block grammar. Declarations as statements.
Composite literals, with the control-header restriction carried as one flag
down the expression parser. Trailing commas. Conversions and type arguments.
`foreign` blocks.

**Semantic analysis.** Two passes over module scope: declare every name,
then resolve, with cycle detection through inline storage only. Block scopes
on a stack. `Type` gains `Int { width, signed }`, `Float { width }`,
`Slice`, `String`, `CString`, `RawPtr`, and `Pointer` to any type; the
record-only check at `PointerBase` goes, and type identity for arrays,
slices, pointers, and `proc` types becomes structural. The constant
evaluator widens to 128 bits with the integer range stated above. The
constant-range compatibility rule replaces the `ByteRange` case of
`assign_kind`; the conversion table replaces the conversion built-ins.
Immutability is a property of a designator's root, found by walking the
selectors, and `&` requires a writable designator. A scan of the body before
slots are assigned finds the roots of `&` operands and slice expressions,
refuses parameters and record-typed fields and elements, and lowers the
promoted locals to a pointer slot filled by an allocation where the
declaration executes; promoted module-level records get a static header. A
type `switch` on a pointer lowers to a read-only temporary that the arms
retype. Integer bit operators and shift checks join
the arithmetic. Foreign procedures are symbols with no body and a
compatibility check on their signature.

**IR.** `Ty` gains the sized integers and `F32`; `Bool` becomes one byte.
A string or slice value is sixteen bytes of storage. As a value parameter
it is two scalar arguments, which open arrays already are and which is also
what a foreign procedure receives; as a `var` parameter it is the address
of the caller's sixteen bytes, so that assigning a new string or slice to
the parameter updates the caller's variable; as a result it is returned
through a hidden out-pointer. `Call` gains the index at which variadic arguments
begin, computed after expansion, together with the promotions applied
there. `Global` gains `extern` and an optional static descriptor header. A
module records the libraries its `foreign` blocks named.

**QBE lowering.** Sub-word loads and stores (`loadsb`, `loadub`, `storeh`,
and so on), sub-word ABI types in signatures and calls, `s` for `f32`, the
`...` marker at the recorded position in variadic calls, and `export`
versus external symbols.

**Runtime.** String concatenation, comparison, and the `string`, `cstring`,
and `[]byte` conversions; slice allocation with the size checks; the
division check gains the `-1` case; the shift check is new. Every
buffer-taking function in `standard.c` is audited for NUL assumptions. The
collector and the check-failure reporting are unchanged.

**Driver.** `.puck` files, `.c` companions compiled with the host `cc`,
`-l` flags from `foreign` strings, `main` and `init` in place of module
bodies, and the removal of the bundled-only private import.

**Library and tests.** The bundled modules are rewritten in Puck. The corpus
is translated program by program; each translation is a test of the
language design as much as of the compiler.

### A sequence

1. Surface syntax, newline rule, precedence, block scope, zero
   initialization, `return`, `break`, `continue`, `for`, `switch`, and
   `init`/`main`. The type system is still Oberon's. The corpus is
   translated and passes.
2. Order-independent module scope and structural identity for the
   non-record composites.
3. Sized integers, `f32`, one-byte `bool`, `^T` to any type, the constant
   range, constant compatibility, the conversion table, the bit operators,
   `&`, roots, and the heap rule.
4. Slices and `string`, with `copy`, `slice`, `len`, and the conversions.
   The tests that exercise open arrays, `CHAR`, and `Strings` are
   translated a second time here.
5. `foreign`, `cstring`, `rawptr`, variadic calls, link libraries, and
   companion C. The bundled modules move onto `foreign` and the private
   interface is removed. The step ends with an ABI suite: C programs,
   compiled by the host `cc`, that call and are called through every shape
   of foreign-compatible signature and compare results.

Each step leaves a working compiler and a passing corpus. The order is
chosen to keep integration risk low, not to avoid rework: a fair share of
the corpus, every test that touches arrays of characters or open arrays, is
translated in step 1 and again in step 4, and that second pass is budgeted
rather than wished away. Step 1 is the largest. Step 5 is the reason for the
rest, and the one whose correctness is judged against another compiler, so
it is the step with a test suite of its own instead of a corpus translation.

## Appendix: Three C Libraries

Three complete programs, each against a library as it ships: zlib 1.3.2,
SQLite 3.53.4, and SDL 3.4.16. The signatures, typedefs, and constants were
checked against the headers of those releases on an LP64 platform. Each
program uses only what this document defines, and each is the whole of what
a programmer writes: no header, no shim, no build script beyond the module
itself. The library name in the `foreign` string is what the driver passes
to the linker. The libraries must already be installed in the linker's
search path.

Together they exercise every piece of the interface: `cstring` in both
directions, a `string` and a slice expanded to pointer and count, `&` on a
promoted local to fill a C out-parameter, a record laid out as a C struct,
a C union treated as a record with a common prefix, a callback, `slice` over
C's memory, `nil` for an optional pointer or procedure, constants adapting
to `u8`, `u32`, and `u64`, and discarded status results.

### zlib: round trip through `compress2`

zlib's `uLong` is C's `unsigned long`, eight bytes on LP64, so it is `u64`;
`Bytef` is `unsigned char`, so a buffer is `^byte`. The source of
`compress2` is declared as a `string` and the source of `uncompress` as a
`[]byte`; each becomes the pointer-and-count pair the C signature has in
that position, and the `level` parameter follows as it does in C. The
expanded count is Puck's `int`; its nonnegative values have the same 64-bit
representation as `uLong` on LP64.

```puck
module Zip

import Out

foreign "z" {
    proc zlibVersion(): cstring
    proc compressBound(sourceLen: u64): u64
    // C: int compress2(Bytef *dest, uLongf *destLen,
    //                  const Bytef *source, uLong sourceLen, int level)
    proc compress2(dest: ^byte, destLen: ^u64, source: string, level: i32): i32
    // C: int uncompress(Bytef *dest, uLongf *destLen,
    //                   const Bytef *source, uLong sourceLen)
    proc uncompress(dest: ^byte, destLen: ^u64, source: []byte): i32
}

const Z_OK = 0
const Z_BEST_COMPRESSION = 9

proc main() {
    var text = ""
    for i in 0..<64 {
        text += "the quick brown fox jumps over the lazy dog\n"
    }

    var packed = new([]byte, int(compressBound(u64(len(text)))))
    var packedLen = u64(len(packed))             // address taken: lives on the heap
    var rc = compress2(&packed[0], &packedLen, text, Z_BEST_COMPRESSION)
    assert(rc = Z_OK)

    var unpacked = new([]byte, len(text))
    var unpackedLen = u64(len(unpacked))
    rc := uncompress(&unpacked[0], &unpackedLen, packed[:int(packedLen)])
    assert(rc = Z_OK)
    assert(string(unpacked[:int(unpackedLen)]) = text)

    Out.String("zlib ")
    Out.String(string(zlibVersion()))
    Out.String(": ")
    Out.Int(len(text), 0)
    Out.String(" bytes -> ")
    Out.Int(int(packedLen), 0)
    Out.String(" bytes and back")
    Out.Ln()
}
```

`&packed[0]` needs no promotion: a slice index is an indirection, so the
root is the heap object the slice views. `&packedLen` promotes the local,
and C writes through the pointer into a heap word the collector keeps.

### SQLite: a table, a query, and a callback

`sqlite3` is an opaque struct in C, so a handle is a `rawptr`, and
`sqlite3_open` fills one through a `^rawptr`. The callback's `char **`
parameters are `^cstring`; `slice` views them as `[]cstring` for the length
SQLite reports. A `nil` callback and a `nil` error pointer are what the C
documentation permits, and the string literals passed as `cstring` are
static data with a terminator.

```puck
module Inventory

import Out, Program

foreign "sqlite3" {
    proc sqlite3_libversion(): cstring
    proc sqlite3_open(filename: cstring, db: ^rawptr): i32
    // C: int sqlite3_exec(sqlite3 *, const char *sql,
    //                     int (*callback)(void *, int, char **, char **),
    //                     void *arg, char **errmsg)
    proc sqlite3_exec(db: rawptr, sql: cstring,
                      callback: proc(arg: rawptr, count: i32,
                                     values, names: ^cstring): i32,
                      arg: rawptr, errmsg: ^cstring): i32
    proc sqlite3_errmsg(db: rawptr): cstring
    proc sqlite3_close(db: rawptr): i32
}

const SQLITE_OK = 0

proc PrintRow(arg: rawptr, count: i32, values, names: ^cstring): i32 {
    var cols = slice(values, int(count))         // programmer-checked: count is SQLite's
    var labels = slice(names, int(count))
    for i, v in cols {
        if i > 0 { Out.String(", ") }
        Out.String(string(labels[i]))
        Out.String("=")
        if v = nil {
            Out.String("NULL")
        } else {
            Out.String(string(v))
        }
    }
    Out.Ln()
    return 0                                     // nonzero would abort the exec
}

proc Exec(db: rawptr, sql: cstring) {
    var rc = sqlite3_exec(db, sql, nil, nil, nil)
    if rc != SQLITE_OK {
        Out.String("sqlite: ")
        Out.String(string(sqlite3_errmsg(db)))
        Out.Ln()
        Program.Exit(1)
    }
}

proc main() {
    Out.String("sqlite ")
    Out.String(string(sqlite3_libversion()))
    Out.Ln()

    var db: rawptr                               // address taken: lives on the heap
    var rc = sqlite3_open(":memory:", &db)
    assert(rc = SQLITE_OK)

    Exec(db, "create table parts (name text, qty integer)")
    Exec(db, "insert into parts values ('bolt', 40), ('nut', 36), ('washer', 0)")

    rc := sqlite3_exec(db, "select name, qty from parts where qty > 0 order by name",
                       PrintRow, nil, nil)
    assert(rc = SQLITE_OK)
    sqlite3_close(db)                            // status discarded
}
```

Output:

```text
sqlite 3.53.4
name=bolt, qty=40
name=nut, qty=36
```

`PrintRow` is a module-level procedure with a foreign-compatible signature,
so its value is the C function pointer SQLite expects. Its `arg` parameter
is unused here; a program that needs state in the callback passes a pointer
to a heap record through it and converts back with a named pointer type.

### SDL3: a window, a rectangle, and the event loop

`SDL_Event` is a 128-byte union whose every member begins with a 32-bit
event type; the C header asserts the size. Puck declares a record with the
common prefix, `type`, `reserved`, and `timestamp`, and pads it to the
union's size, which is the document's rule that a union is bytes with a
known layout. `SDL_FRect` is four `f32` fields, and the record matches the
struct. SDL3 returns `bool` from most calls, and `bool` is one byte on both
sides. `SDL_WindowFlags` is a 64-bit type in SDL3 and `SDL_InitFlags` a
32-bit one; the constants adapt to each.

```puck
module Bounce

import Out, Program

type FRect = record {
    x, y, w, h: f32
}

// SDL_Event is a 128-byte union; every member starts with a 32-bit type.
type Event = record {
    kind: u32
    reserved: u32
    timestamp: u64
    padding: [112]byte
}

foreign "SDL3" {
    proc SDL_Init(flags: u32): bool
    proc SDL_Quit()
    proc SDL_GetError(): cstring
    proc SDL_CreateWindow(title: cstring, w, h: i32, flags: u64): rawptr
    proc SDL_DestroyWindow(window: rawptr)
    proc SDL_CreateRenderer(window: rawptr, name: cstring): rawptr
    proc SDL_DestroyRenderer(renderer: rawptr)
    proc SDL_SetRenderDrawColor(renderer: rawptr, r, g, b, a: u8): bool
    proc SDL_RenderClear(renderer: rawptr): bool
    proc SDL_RenderFillRect(renderer: rawptr, rect: ^FRect): bool
    proc SDL_RenderPresent(renderer: rawptr): bool
    proc SDL_PollEvent(event: ^Event): bool
    proc SDL_Delay(ms: u32)
}

const SDL_INIT_VIDEO = 0x20
const SDL_EVENT_QUIT = 0x100

proc Fail(what: string) {
    Out.String(what)
    Out.String(": ")
    Out.String(string(SDL_GetError()))
    Out.Ln()
    Program.Exit(1)
}

proc main() {
    if not SDL_Init(SDL_INIT_VIDEO) { Fail("SDL_Init") }
    var window = SDL_CreateWindow("Puck", 640, 480, 0)   // fixed size: the bounce uses 640
    if window = nil { Fail("SDL_CreateWindow") }
    var renderer = SDL_CreateRenderer(window, nil)   // nil: the default driver
    if renderer = nil { Fail("SDL_CreateRenderer") }

    var box = FRect{x: 0.0, y: 200.0, w: 80.0, h: 80.0}   // address taken below
    var dx: f32 = 4.0
    var event: Event                                      // address taken below
    var running = true
    for running {
        for SDL_PollEvent(&event) {
            if event.kind = SDL_EVENT_QUIT { running := false }
        }
        box.x += dx
        if box.x < 0.0 or box.x + box.w > 640.0 { dx := -dx }

        SDL_SetRenderDrawColor(renderer, 20, 20, 40, 255)
        SDL_RenderClear(renderer)
        SDL_SetRenderDrawColor(renderer, 240, 200, 60, 255)
        SDL_RenderFillRect(renderer, &box)
        SDL_RenderPresent(renderer)
        SDL_Delay(16)
    }

    SDL_DestroyRenderer(renderer)
    SDL_DestroyWindow(window)
    SDL_Quit()
}
```

`box` and `event` are whole record variables, so `&` is permitted on them;
both are promoted to the heap, where the descriptor header sits before the
object and the pointer C receives is the object itself. On the supported
macOS and Linux platforms, SDL3 does not require its own `main`: a program
that owns the entry point calls `SDL_Init` directly, which is what `proc main`
does.
