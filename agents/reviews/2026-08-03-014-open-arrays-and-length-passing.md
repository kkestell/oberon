# Review: Open arrays and length passing

Read against an uncommitted working tree on top of commit `c9191e6` (`add pointers nil and garbage collected allocation`). The slice is the one described in [the Slice 14 plan](../plans/2026-08-03-014-open-arrays-and-length-passing.md).

## Verdict

No behavioural defect was found. The open-array form, the recursive compatibility rule, the address-plus-lengths calling convention, dynamic row-major strides, dynamic `LEN`, string actuals, open character-array comparison, read-only propagation, and the checked prefix copy all do what the plan describes, and all of them survived probing.

Four findings were recorded and fixed. All four are about comments, duplicated code, and a missing test; none of them changed what a program does. One further observation is recorded as deliberately declined rather than fixed.

Code generation for programs that use no open array is byte-identical to the pre-slice compiler across all 110 existing corpus and failure modules.

## What was confirmed

### The calling convention

`build/OpenArrays.ssa` shows one long address followed by one word length per open dimension. A value formal and a `VAR` formal have the same machine signature: `Sum(a: ARRAY OF INTEGER)` becomes `function w $OpenArrays.Sum(l %.t0, w %.t1)` and `SetFirst(VAR a: ARRAY OF INTEGER; n: INTEGER)` becomes `function $OpenArrays.SetFirst(l %.t0, w %.t1, w %.t2)`. Neither prologue copies array data.

Two names in one section expand independently. `Lengths(a, b: ARRAY OF INTEGER)` becomes `function $OpenArrays.Lengths(l %.t0, w %.t1, l %.t2, w %.t3)`, and the call site passes the two actuals' separate lengths.

Source arity still counts one formal per declared name. `One(ints, ints)` reports `wrong number of arguments: expected 1, found 2` rather than counting the machine arguments.

### Strides

For a three-dimensional open formal the outer stride multiplies both remaining incoming lengths by the terminal element size, and each selected dimension drops one factor. `Inspect3` in `build/OpenArrayDimensions.ssa` computes the outer step as `mul %.x, 4` then `mul` by `%.t2` then `mul` by `%.t3`, the middle step as `mul %.x, 4` then `mul` by `%.t3`, and the inner step as `mul %.x, 4`. The bounds check for each dimension is emitted before that dimension's widening and multiplication.

A named fixed-array base contributes its complete static size instead of another hidden value. `Mixed(a: ARRAY OF Row)` forwards to `Inspect2(a: ARRAY OF ARRAY OF INTEGER)` as `call $OpenArrayDimensions.Inspect2(l %.t0, w %.t1, w 3)`: one incoming dynamic length and one immediate from `Row`.

Fixed indexing is unchanged. The module body's `cube[1, 2, 3]` still emits `mul %.x15, 48`, `mul %.x16, 16`, `mul %.x17, 4` against immediate bounds.

### The copy check

Both failing copies check before they move anything. `OpenArrayCopyTooLong.Copy` emits `call $oberon_check_array_copy(w %.t1, w 2)` and only then widens the accepted length, multiplies by the element size, and calls `oberon_copy`. `OpenArrayStringTooLong.Copy` emits `call $oberon_check_array_copy(w 4, w %.t1)` before a copy of the constant four bytes, so a string whose characters fit but whose terminator does not is rejected. Because the check exits the process, the emitted order is enough to show that no destination byte changes first.

### Lengths never escape their procedure

An open formal's lengths are temporaries of the procedure that received them. A nested procedure that referred to one would produce a temporary name that does not exist in its own function. This cannot happen, because a nested procedure already cannot name an enclosing procedure's parameters at all. This program is rejected:

```oberon
MODULE Probe6;
VAR n: INTEGER;
    g: ARRAY 2 OF INTEGER;
PROCEDURE Outer(a: ARRAY OF INTEGER);
  PROCEDURE Inner;
  BEGIN n := LEN(a)
  END Inner;
BEGIN Inner
END Outer;
BEGIN Outer(g)
END Probe6.
```

It reports `'a' is not accessible: a nested procedure cannot use the variables or parameters of an enclosing procedure`.

### Read-only propagation

A value open formal stays unwritable through every derived place, and a `VAR` open formal stays writable. This program reports exactly three errors, on the first three calls in `Bad`, and accepts the last three:

```oberon
MODULE Probe8;
TYPE Row = ARRAY 3 OF INTEGER;
     Rec = RECORD f: Row END;
VAR rows: ARRAY 2 OF Row;
    recs: ARRAY 2 OF Rec;
PROCEDURE TakeRow(VAR r: Row); END TakeRow;
PROCEDURE TakeRec(VAR r: Rec); END TakeRec;
PROCEDURE Open(a: ARRAY OF INTEGER); END Open;
PROCEDURE VarOpen(VAR a: ARRAY OF INTEGER); END VarOpen;
PROCEDURE Bad(v: ARRAY OF Row; w: ARRAY OF Rec; VAR ok: ARRAY OF Row);
BEGIN
  TakeRow(v[0]);
  TakeRec(w[0]);
  VarOpen(v[0]);
  TakeRow(ok[0]);
  VarOpen(ok[0]);
  Open(w[0].f)
END Bad;
BEGIN Bad(rows, recs, rows)
END Probe8.
```

Each rejection is `argument 1 is read-only`. Selecting a row, a record, or a field from a value open formal keeps it read-only, and selecting from a `VAR` open formal keeps it writable.

### Element types and interior addresses

Open formals work over `BYTE`, `REAL`, `SET`, `BOOLEAN`, pointers, named fixed arrays, and records, at every rank probed. A heap record's array field can be passed to both a value and a `VAR` open formal, and the resulting interior address survives a hundred thousand intervening allocations, which is what Slice 13's arbitrary-interior-pointer setting is there for.

Open character arrays compare against strings, against fixed character arrays, and against each other, including a row selected from a two-dimensional open formal. String assignment through a writable open character array checks the characters and the terminator against the incoming length.

### Constant `LEN`

`LEN` of an open dimension is not a constant, and a fixed dimension reached through an open prefix still is. `is_const_expr` decides this from `Type::array`, which is `None` for an open array, so `LEN` of an open formal reports as nonconstant rather than as an invalid constant. `LEN(a)` in a constant declaration and `LEN(a[0])` as another array's length are both rejected once each, while `CONST Width = LEN(mixed[i])` folds to `Row`'s declared length without evaluating `i`. `OpenArrayDimensions.Mod` pins that by calling `Mixed(rows, 27)` with an index that would be out of range if it were ever used.

The previous slice's fix for a procedure call inside a required-constant `LEN` still holds through an open prefix. `CONST A = LEN(m[Index() + 1])`, where `m` is `ARRAY OF Row`, still reports `constant expression contains a procedure call`.

## Findings

### Low: the IR comment on `Index` still described open arrays as future work

`Inst::Index` in `src/ir.rs` explained its length operand by saying "an open array will supply a dynamic length here without changing the address rule". That sentence was written before this slice and is now describing the present. Rewritten to say that an open dimension's length exists only as an incoming value that no allocation records. Fixed.

### Low: `CheckArrayCopy` carried no comment

Every neighbouring instruction in `src/ir.rs` explains why it exists. The new capacity check had nothing, and the reason it is a separate instruction rather than part of the copy is exactly the interesting part. A comment now says that separating it is what lets it fail without moving a byte. Fixed.

### Low: two identical assignment arms

`lower_assign` in `src/sema.rs` matched `AssignKind::StringCopy` and `AssignKind::OpenStringCopy` in two arms with identical bodies, which invites the reader to look for a difference that is not there. The real difference is inside `copy_string`, which reads the destination type to decide whether the fit is settled at compile time or at run time. The two arms are merged into one pattern with a comment saying where the difference lives. Fixed.

### Low: the parser test omitted an exported procedure

The plan listed exported procedures among the formal-type shapes the parser test should cover. `open_array_formals` covered one, two, and three open dimensions, a named fixed-array base, mixed value and `VAR` sections, several names in one section, and a nested procedure, but not an export mark. Added `PROCEDURE R*(text: ARRAY OF CHAR)` to the same test. Fixed.

## Declined

### A constant-folding error inside an open index is not reported

An index expression that the compiler could fold, but that is invalid, is a source error on a fixed array and silently accepted on an open one:

```oberon
MODULE Probe3;
VAR fixed: ARRAY 3 OF INTEGER;
    n: INTEGER;
PROCEDURE P(a: ARRAY OF INTEGER);
BEGIN n := a[1 DIV 0]
END P;
PROCEDURE Q;
BEGIN n := fixed[1 DIV 0]
END Q;
BEGIN P(fixed); Q
END Probe3.
```

The only diagnostic is `constant DIV or MOD by zero` at the line in `Q`. The line in `P` compiles.

This looks like an asymmetry the slice introduced, and it is not worth removing. The fixed-array path folds its index because it has to: it needs the value to check it against a declared length, and folding is what surfaces the division. The open path has no declared length to check against, so it never folds, and it therefore behaves like every other expression context. `n := 1 DIV 0` on its own also compiles today. Making the open index fold would give it a diagnostic that plain assignment does not have, which trades one asymmetry for another. The fixed index is the outlier here, and it earns its outlier status.

## Notes for later slices

Two fixed array types that were written separately print the same in a diagnostic, so passing an `ARRAY 1 OF B` to a formal of `ARRAY OF A`, where `A` and `B` are separately declared but identical, reports `argument 1 has type ARRAY 1 OF ARRAY 2 OF INTEGER, expected ARRAY OF ARRAY 2 OF INTEGER`. The rejection is right and the message is unhelpful. The plan states that this slice does not add declared names to fixed-array diagnostics, so this is not a finding, but the open-array rank rule makes the confusing case easier to write than it used to be.

Slice 16 compares procedure signatures. It must compare open rank and the value-versus-`VAR` mode, not just the terminal types, because two formals with the same base and different rank now have the same printed base.
