MODULE ArrayCopy;
  (* Report 9.1: an array assignment copies the value. The destination owns
     its bytes afterwards, so changing the source cannot reach it. The types
     that may take part are the same type, which here means one named type,
     an alias of it, and two variables written in one declaration. *)
  IMPORT Out;
  TYPE Row = ARRAY 3 OF INTEGER;
       Alias = Row;
  VAR a, b: Row;
      viaAlias: Alias;
      pair, other: ARRAY 2 OF INTEGER;
      grid: ARRAY 3 OF Row;
      empty, alsoEmpty: ARRAY 0 OF INTEGER;
      rows: ARRAY 2 OF ARRAY 0 OF INTEGER;
      order: INTEGER;

  PROCEDURE Pick(n: INTEGER): INTEGER;
  BEGIN INC(order); Out.Int(order, 0); Out.Int(n, 2); Out.Ln
  RETURN n
  END Pick;

  PROCEDURE Nothing;
    (* Zero-length arrays in an activation record. They reserve nothing, the
       assignment between them copies nothing, and neither fact needs a
       hidden element or a minimum payload byte. *)
    VAR none, alsoNone: ARRAY 0 OF INTEGER;
  BEGIN
    none := alsoNone;
    Out.Int(LEN(none), 0); Out.Int(LEN(alsoNone), 4); Out.Ln
  END Nothing;

  PROCEDURE ShowRow(label, first, second, third: INTEGER);
  BEGIN
    Out.Int(label, 0); Out.Int(first, 4); Out.Int(second, 4);
    Out.Int(third, 4); Out.Ln
  END ShowRow;

BEGIN
  a[0] := 1; a[1] := 2; a[2] := 3;

  (* A copy, then a change to the source. The destination keeps what it was
     given, which is what makes this an assignment and not an alias. *)
  b := a;
  a[0] := 99;
  ShowRow(1, b[0], b[1], b[2]);
  ShowRow(2, a[0], a[1], a[2]);

  (* An alias is the same type, so this is an ordinary assignment. *)
  viaAlias := b;
  viaAlias[2] := 30;
  ShowRow(3, viaAlias[0], viaAlias[1], viaAlias[2]);
  ShowRow(4, b[0], b[1], b[2]);

  (* Two anonymous arrays declared in one identifier list share the one type
     that declaration resolved. *)
  pair[0] := 5; pair[1] := 6;
  other := pair;
  pair[1] := 60;
  Out.Int(other[0], 0); Out.Int(other[1], 4);
  Out.Int(pair[0], 4); Out.Int(pair[1], 4); Out.Ln;

  (* Self-assignment copies a region onto itself and must leave it alone. *)
  a := a;
  ShowRow(5, a[0], a[1], a[2]);

  (* A selected row is an array designator too. The destination is resolved
     before the source, so Pick reports 2 before 0. *)
  grid[0][0] := 10; grid[0][1] := 11; grid[0][2] := 12;
  order := 0;
  grid[Pick(2)] := grid[Pick(0)];
  ShowRow(6, grid[2][0], grid[2][1], grid[2][2]);

  (* A zero-length array has a length and an assignment, and both of them are
     as empty as they sound. *)
  empty := alsoEmpty;
  Out.Int(LEN(empty), 0); Out.Int(LEN(alsoEmpty), 4); Out.Ln;

  (* An array of zero-length arrays. The outer dimension has two real elements
     of no size at all, so selecting one is a checked index and copying one is
     a copy of nothing. *)
  rows[1] := rows[0];
  Out.Int(LEN(rows), 0); Out.Int(LEN(rows[1]), 4); Out.Ln;
  Nothing
END ArrayCopy.
