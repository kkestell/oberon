# About Oberon-07

Oberon-07 is a small, statically typed, procedural language. It is the last in the line that runs from Algol through Pascal and Modula-2, and it is by a wide margin the smallest of them. The whole language is defined in seventeen pages.

This document is an introduction, not a definition. It tries to explain what the language is like and why it is shaped the way it is. The normative definition is Niklaus Wirth's *The Programming Language Oberon (Revised Oberon-07)*, May 2016, kept in this repository at [references/oberon07-report.pdf](../references/oberon07-report.pdf) and referred to throughout as the Report. When this document and the Report disagree, the Report is right.

## The shape of a program

Here is a complete Oberon program.

```oberon
MODULE Squares;
  IMPORT Out;
  CONST N = 10;
  VAR i, total: INTEGER;

  PROCEDURE Square(x: INTEGER): INTEGER;
  BEGIN
  RETURN x * x
  END Square;

BEGIN
  total := 0;
  FOR i := 1 TO N DO
    total := total + Square(i)
  END;
  Out.Int(total, 0); Out.Ln
END Squares.
```

It prints `385`.

Several things in that example are worth pointing at now, because they recur everywhere.

A program is a module, and a module is a single file. There is no separate interface or header file that repeats the declarations. The `IMPORT` line is the whole dependency declaration, and `Out.Int` is how you name something that came from an import.

Declarations come in a fixed order: constants, then types, then variables, then procedures. You cannot interleave them, and you cannot declare a variable in the middle of a statement sequence. Every declaration section is optional, so a module with nothing to declare simply omits them.

The `RETURN` in `Square` is not a statement. It is a clause that belongs to the procedure body and may appear only at the end of it, after any statements. This is why it sits at the same indentation as `BEGIN` in the example above rather than inside the body. A procedure has exactly one result expression or none at all.

The module's name is repeated after the final `END`, and so is the procedure's. The Report requires this. It costs a little typing and it makes a truncated or mis-nested file fail immediately rather than mysteriously.

The final `END Squares` is followed by a period. That period is part of the grammar and marks the end of the compilation unit.

The statements between `BEGIN` and `END` at the module level are the module's initialization. They run once, when the module is loaded, and every module a program uses gets its own.

## Where the language came from

Oberon was defined in 1988 and 1990 at ETH Zürich. It came out of Modula-2, and the Report's own introduction names its principal new feature: type extension, the ability to build a new record type on an existing one. Everything else about Oberon relative to Modula-2 was subtraction. The language was designed alongside an operating system of the same name, written in it, and that is the important context for its size. The language had to be enough to write an operating system and a compiler, and it was not permitted to be more than that. Simplicity here is not a matter of taste. It was a budget.

Wirth revised the language in 2007, and revised the revision through 2013 and 2016. What is remarkable about the 2007 revision is its direction. Almost every language that survives twenty years grows. Oberon-07 shrank, and Wirth wrote a short note explaining each removal. That note is *Differences between Revised Oberon and Oberon*, and the reasoning in it tells you more about the language's character than a feature list would.

The `LOOP` statement and its `EXIT` went away. Wirth's argument was structural: the properties of a structured statement should follow from the properties of its parts, and a loop whose exits are scattered through its body anywhere breaks that. As compensation, `WHILE` gained `ELSIF` branches, giving it the shape of Dijkstra's guarded repetition. His favourite example was the subtractive Euclidean algorithm.

```oberon
WHILE m > n DO m := m - n
ELSIF n > m DO n := n - m
END
```

The loop repeats as long as any guard is true and stops when all of them are false. Note that the branches use `DO`, not `THEN`, which is how you tell a `WHILE` from an `IF` when you are skimming.

`RETURN` stopped being a statement for the same reason. A result expression buried somewhere inside a procedure body is syntactically disconnected from the procedure's declared result type, which makes "does this procedure always produce a result?" a question a reader has to answer by searching. Moving the result to the end of the body makes it a question a reader answers by looking in one place.

The `WITH` statement went away. The 2013 revision then extended `CASE` so that its selector may be a record or a pointer, which covers what `WITH` was for.

Assignment of whole arrays and records arrived, and the standard procedure `COPY` that used to be needed for it was discarded. Wirth's earlier reasoning had been that copying a large structure is expensive enough that a programmer should see it happening. He changed his mind: `dst := src` now works for arrays and records, and it means what it says.

The integer size variants went away, along with the numeric type inclusion hierarchy that let a smaller integer type stand in for a larger one. Oberon-07 has one integer type and one real type. Several standard procedures went with them, including `MIN`, `MAX`, `HALT`, `CAP`, and `ENTIER`. If you come from Oberon-2 or Pascal and reach for `MAX(INTEGER)`, it is not there.

## The module is the only unit of structure

Oberon has one mechanism for organizing a program, and it is the module. There are no classes, no namespaces, no packages, no generics, no macros, and no preprocessor. When you want to draw a line inside a program, you draw it with a module boundary, and that is the only line available.

A module names what it wants from other modules, and it marks what it offers. The mark is an asterisk on the declared identifier.

```oberon
MODULE Counters;
  TYPE Counter* = RECORD
    count*: INTEGER;
    limit: INTEGER
  END;

  VAR Created*: INTEGER;

  PROCEDURE Init*(VAR c: Counter; limit: INTEGER);
  BEGIN
    c.count := 0; c.limit := limit; INC(Created)
  END Init;

BEGIN
  Created := 0
END Counters.
```

Everything without an asterisk is private, and privacy is the default. That includes record fields: `count` is visible to clients and `limit` is not, even though the record type itself is exported. A client can read and write `count` on a `Counters.Counter`, and cannot see that `limit` exists at all.

An exported variable is exported read-only. A client may read `Counters.Created` and may not assign to it. This was one of the 2007 changes, and Wirth's justification is worth repeating because it explains a small annoyance: the guideline is to export only constant things, but strictly following it forces you to write a trivial accessor procedure whenever a client needs to see a number. Read-only export is the compromise. The variable appears to the client as a constant.

A client imports by name, and may rename on import.

```oberon
MODULE App;
  IMPORT Out, C := Counters;
  VAR c: C.Counter;
BEGIN
  C.Init(c, 3);
  Out.Int(c.count, 0); Out.Ln
END App.
```

`C := Counters` binds the local name `C` to the module `Counters`. This is the only aliasing the language has, and it exists so a module with a long name does not force long qualified names on its clients.

Two things follow from the module being the only structuring mechanism. The first is that separate compilation is straightforward: a client needs to know only the exported part of what it imports, which the compiler can derive from the source. The second is that the language has no way to talk about a group of modules. There is no notion of a library, a package, or a visibility scope wider than one module and narrower than the whole program.

## The types are few, and conversions are written down

There are six basic types. `BOOLEAN` holds `TRUE` and `FALSE`. `CHAR` holds characters of a standard character set. `INTEGER` holds integers and `REAL` holds real numbers. `BYTE` holds the integers from 0 through 255, and is the one type with an implicit relationship to another: `BYTE` and `INTEGER` are compatible in both directions. `SET` holds sets of small non-negative integers, up to a limit the implementation chooses.

There are two ways to build a structured type, and only two. An array is a fixed number of elements of one type. A record is a fixed number of named fields of possibly different types. Beyond those there are pointers, which must point at records, and procedure types.

The rule that gives the type system its character is the assignment rule: the type of the expression must be *the same as* the type of the destination. Not compatible, not convertible, the same. There are exactly four exceptions, and they are worth learning because they are the only places in the language where the two sides of an assignment may have different types.

`NIL` may be assigned to any pointer or procedure variable. A string may be assigned to any character array long enough to hold it, and a one-character string may also be assigned to a `CHAR`. A record may be assigned to a variable of one of its base types. An open array may be assigned to an array with the same element type.

Everything else you write yourself. `FLT` turns an `INTEGER` into a `REAL` and `FLOOR` turns a `REAL` into an `INTEGER`. `ORD` gives you the ordinal of a `CHAR`, a `BOOLEAN`, or a `SET`, and `CHR` goes back the other way. There is no widening, no promotion, and no context in which the compiler inserts a conversion you did not write. An `INTEGER` and a `REAL` cannot be the two operands of the same arithmetic operator; one of them has to be converted in the source first.

This is strict enough to be annoying in small doses and it buys something specific. When you read an Oberon expression, the type of every operand is the declared type of that operand. Nothing was silently converted on the way in.

`SET` deserves a note, because it is not the general set type its name suggests. In practice it is a machine word used as a bit vector, with notation pleasant enough that you forget that is what it is.

```oberon
flags := {0, 2, 4 .. 6};
IF 5 IN flags THEN Report END;
flags := flags - {2};
INCL(flags, 9)
```

Union is `+`, difference is `-`, intersection is `*`, and symmetric difference is `/`. A unary minus on a set is its complement. Membership is `IN`. The range form `{4 .. 6}` means `{4, 5, 6}`, and a range whose lower bound exceeds its upper bound is the empty set rather than an error.

Strings are the other place where the type system bends. There is no string type. A string literal is a constant, and text lives in a character array. Assigning a string to a character array copies the characters and appends one null character, so the array has to be strictly longer than the text it holds, and reading the text back means stopping at the null.

```oberon
VAR name: ARRAY 16 OF CHAR;
    i: INTEGER;
...
name := "Oberon";
i := 0;
WHILE name[i] # 0X DO Out.Char(name[i]); INC(i) END
```

The literal `0X` is a one-character string given by the character's ordinal in hexadecimal, so `0X` is the null character. This is the same notation as `22X` for a quote mark, which you need because a string cannot contain the quote that delimits it. Oberon has no escape sequences.

The comparison operators work on character arrays as well as on numbers and single characters, comparing them lexically, so `name < other` is a legal expression and needs no procedure call.

## Type extension is the one big idea

Everything above is subtraction. Type extension is the thing Oberon added, and the Report leads with it.

A record type may be declared as an extension of another record type. The extension has all of the base type's fields plus its own.

```oberon
TYPE
  Node = RECORD
    key: INTEGER
  END;

  NamedNode = RECORD (Node)
    name: ARRAY 32 OF CHAR
  END;
```

A `NamedNode` is a `Node`. That claim has three consequences, and they are the whole machinery.

A `NamedNode` may be assigned to a `Node` variable, which copies the base fields and discards the rest. A `NamedNode` variable may be passed to a parameter declared `VAR n: Node`. That second point is a real relaxation: a variable parameter otherwise demands an actual of exactly its declared type, and a record is the one case where a base type is accepted instead. And within such a procedure, you can ask what you were actually given.

```oberon
PROCEDURE Show(VAR n: Node);
BEGIN
  Out.Int(n.key, 0);
  IF n IS NamedNode THEN
    Out.Char(n(NamedNode).name[0])
  END
END Show;
```

`n IS NamedNode` is a type test and yields a Boolean. `n(NamedNode)` is a type guard: it asserts that `n` really is a `NamedNode` and gives you a designator of that type to select from. A guard that turns out to be false aborts the program. Both are restricted to subjects whose actual type can differ from their declared type, which means a variable parameter of record type or a pointer.

Pointers inherit the relation. If `NamedNode` extends `Node`, then a pointer bound to `NamedNode` extends a pointer bound to `Node`, and everything above applies to the pointers too. The `CASE` statement can select on a record or a pointer as well as on an integer or a character, which gives you a multi-way form of the same test.

What Oberon does not add alongside extension is methods. There are no type-bound procedures, no virtual dispatch, and no inheritance of behaviour — only of fields. When you want dynamic dispatch, you put a procedure-typed field in the record and assign to it. That is a deliberate choice rather than an omission: it makes the dispatch table something you can see and read in the source, at the cost of writing it out yourself.

Procedure types are constrained in a way that is easy to trip over. Only a procedure declared at module level can be assigned to a procedure variable. A procedure nested inside another procedure cannot, and neither can a predefined procedure like `ABS`. There are no closures.

## Statements, and the single exit

The statement forms are assignment, procedure call, `IF`, `CASE`, `WHILE`, `REPEAT`, and `FOR`. That is the complete list. There is no `LOOP`, no `break`, no `continue`, no `goto`, and no early `RETURN`. Once a statement sequence starts, it finishes.

Semicolons separate statements rather than terminating them, and the empty statement is legal, so a stray semicolon before an `END` is harmless. There is no `BEGIN` or braces around the arms of an `IF` — the keywords `THEN`, `ELSE`, and `END` do that work, and every structured statement is closed by its own `END`.

```oberon
IF t = NIL THEN
  NEW(t); t.key := key
ELSIF key < t.key THEN
  Insert(t.left, key)
ELSE
  Insert(t.right, key)
END
```

`CASE` has no `ELSE` branch. A label is an integer literal, a single-character string, or a name — either a declared constant or, in the type form, a type. Labels may be written as ranges with `..`. A selector value matching no label is not something the Report defines, which in practice means implementations abort.

One consequence of a label being a *name* rather than an expression is that a negative label has to be declared as a constant first. `-1` is an expression, so it cannot appear as a label, but `Mid = -1` in the `CONST` section can.

`FOR` is narrower than it looks. The control variable and both bounds must be `INTEGER`, and the step after `BY` must be a constant expression, so a loop's stride is always visible in the source.

Parameters come in two kinds. A variable parameter, marked `VAR`, stands for the caller's variable. A value parameter of a basic type is a local variable holding the actual's value, so the procedure may assign to it freely without affecting the caller.

A value parameter of a *structured* type is different, and this is the subtle corner of the language. It cannot be assigned to, and neither can any of its fields or elements. That restriction is exactly what lets an implementation pass a large array or record by reference instead of copying it, which was the point of the 2007 change. So a structured value parameter is a read-only view of the caller's variable rather than a snapshot of it, and if the same variable also reaches the procedure through a `VAR` parameter, a write through one is visible through the other.

An array parameter can decline to fix its length.

```oberon
PROCEDURE Sum(a: ARRAY OF INTEGER): INTEGER;
  VAR i, total: INTEGER;
BEGIN
  total := 0;
  FOR i := 0 TO LEN(a) - 1 DO total := total + a[i] END
RETURN total
END Sum;
```

`ARRAY OF INTEGER` with no length is an open array, and it accepts an array of any length. `LEN` reports the length it actually received. This is the only place where a type's size is not known at compile time, and open arrays exist only as parameter types — you cannot declare a variable, a field, or a pointer base of one.

A procedure's result can be neither an array nor a record. This surprises people, and the workaround is a `VAR` parameter that the procedure fills in.

## Failure, and memory

Oberon has no exceptions. There is no `try`, no error type, no result convention in the language, and no way to recover from a failed check. A failed type guard aborts. `ASSERT(cond)` aborts if the condition is false, and it is also how you say `HALT`, by writing `ASSERT(FALSE)`. Errors that a program expects to handle are handled by ordinary values that the program tests.

Memory is garbage collected. `NEW(p)` allocates a record for the pointer `p` to reference, and there is no `DISPOSE`. The Report says that a failed allocation leaves `p` equal to `NIL`, which is the closest thing to an error signal in the language.

```oberon
TYPE
  Tree = POINTER TO Node;
  Node = RECORD
    key: INTEGER;
    left, right: Tree
  END;
```

Two details there matter. A pointer must be bound to a record type — you cannot have a pointer to an integer or to an array. And `Node` is named before it is declared, which is legal specifically so that a record and the pointer type that references it can refer to each other. The forward reference has to be resolved in the same scope.

Field selection through a pointer does not need an explicit dereference. `t.key` means `t^.key`; the dot implies the dereference. The caret form still exists and you will see it written out occasionally for emphasis, but the short form is idiomatic and is what the Report's own examples use.

## Reading Oberon code

A handful of things about the notation are worth knowing before you read much of it.

Keywords are written in capitals, and this is not a convention — the language is case sensitive and the reserved words consist of capital letters. `end` is an ordinary identifier. Identifiers themselves are letters and digits only, starting with a letter, so there are no underscores and Oberon code tends to be written in camel case.

The operators will surprise you in two places. Inequality is `#`, not `!=`. Conjunction is `&`, disjunction is `OR`, and negation is `~`, so a Boolean expression mixes a symbol and two words. Both `&` and `OR` are short-circuiting.

Comparison binds *loosely*, more loosely than arithmetic and Boolean operators, so a compound condition needs parentheses around each comparison:

```oberon
IF (0 <= i) & (i < 100) THEN Handle(i) END
```

The parentheses are not optional. `&` binds more tightly than `<=` does, so without them the parser reads `0 <= (i & i)` and then finds a second `<` it has no rule for. An Oberon expression may contain at most one relational operator, so leaving the parentheses out is a syntax error rather than a wrong answer. This is the most common mistake a newcomer makes.

Unary minus is looser still. It attaches to the whole simple expression rather than just to the first factor, which means `-7 DIV 2` is `-(7 DIV 2)`, or -3. Write `(-7) DIV 2` when you want -4.

`DIV` and `MOD` are floored, not truncated. The Report defines them by `x = q*y + r` with `0 <= r < y`, so the remainder is never negative for a positive divisor: `(-7) DIV 2` is -4 and `(-7) MOD 2` is 1. C and most languages with the same operators truncate instead, and the floored definition is the more useful one when you are computing an index.

Braces mean sets, not blocks. `{}` is the empty set.

Comments are `(*` and `*)`, and they nest, so commenting out a region that already contains a comment works.

Numbers have two notations worth recognizing. An integer with an `H` suffix is hexadecimal, so `100H` is 256. A real number always contains a decimal point, and `E` in a real literal means "times ten to the power of", so `4.567E8` is 456700000.

## What the Report leaves to the implementation

The Report is seventeen pages partly because it declines to answer questions it considers none of its business. Its introduction says so outright: what remains unsaid is mostly left so intentionally, either because it follows from stated rules or because saying it would restrict implementations unnecessarily.

So the size of an `INTEGER`, the precision of a `REAL`, the largest element a `SET` can hold, and the character set behind `CHAR` are all decisions the implementation makes. A few finer points are genuinely open too, including what happens when a `CASE` selector matches no label, and whether two separately written array types with the same shape count as the same type.

This compiler's answers are recorded in [docs/dev/architecture.md](dev/architecture.md), and the reasoning behind the ones that required a judgement call is in the slice reviews under [docs/dev/reviews/](dev/reviews/).

## What is not there

It is worth stating the absences plainly, because a language this small is more usefully described by its boundary than by its contents.

There are no exceptions, no generics, no operator overloading, no interfaces or traits, no type inference, no variadic procedures, no default parameter values, no nested modules, and no concurrency. There is no string type, no dynamic array, no hash table, and no iterator protocol. There is no input or output in the language at all — `Out.Int` in the first example comes from a module, not from the language, and a program that does not import such a module has no way to produce a byte.

The language's answer to nearly all of that is the same: write it out. A dispatch table is a record with procedure fields. A growable array is a record holding a pointer and a length, with procedures that manage it. A generic container is a container of pointers to a base record type, with a type guard at the point of use. Oberon does not make these things easy so much as it makes them visible, and whether that trade is worth it depends on what you are building and on how much you want to be able to read the whole thing.

What you get in exchange is a language you can hold in your head. The grammar fits on two pages. There are twenty-four predefined identifiers. A compiler for it is a project one person can finish, which is exactly why this repository exists.

## Where to look next

The Report, at [references/oberon07-report.pdf](../references/oberon07-report.pdf), is short and readable, and is the right next thing to read. Chapters 6 through 11 are the substance, and section 10.2 is the complete list of what the language gives you without an import.

The grammar is extracted for convenient reference at [references/oberon07-grammar.ebnf](../references/oberon07-grammar.ebnf), rearranged into a top-down reading order.

For worked Oberon, the compiler's test corpus in [tests/corpus/](../tests/corpus/) is a few dozen small modules, each exercising one part of the language with its expected output recorded next to it. The modules carry comments citing the Report section they pin down, so they double as annotated examples.

This compiler does not implement all of Oberon-07 yet. Source using a part of the language it has not reached is rejected with a diagnostic saying so, rather than miscompiled, and [docs/dev/roadmap.md](dev/roadmap.md) tracks what is left.
