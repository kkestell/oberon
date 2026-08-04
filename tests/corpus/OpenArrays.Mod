MODULE OpenArrays;
IMPORT Out;
TYPE Row = ARRAY 3 OF INTEGER;
     Node = RECORD n: INTEGER END;
     Link = POINTER TO Node;
VAR calls: INTEGER;
    empty: ARRAY 0 OF INTEGER;
    one: ARRAY 1 OF INTEGER;
    many: ARRAY 4 OF INTEGER;
    table: ARRAY 2 OF Row;
    links: ARRAY 2 OF Link;

PROCEDURE Sum(a: ARRAY OF INTEGER): INTEGER;
VAR i, total: INTEGER;
BEGIN
  i := 0; total := 0;
  WHILE i < LEN(a) DO total := total + a[i]; INC(i) END
RETURN total
END Sum;

PROCEDURE Lengths(a, b: ARRAY OF INTEGER);
BEGIN Out.Int(LEN(a), 0); Out.Char(":"); Out.Int(LEN(b), 0); Out.Ln
END Lengths;

PROCEDURE SetFirst(VAR a: ARRAY OF INTEGER; n: INTEGER);
BEGIN a[0] := n
END SetFirst;

PROCEDURE Forward(a: ARRAY OF INTEGER): INTEGER;
  PROCEDURE Nested(a: ARRAY OF INTEGER): INTEGER;
  BEGIN RETURN Sum(a) END Nested;
BEGIN RETURN Nested(a) END Forward;

PROCEDURE ForwardVar(VAR a: ARRAY OF INTEGER; n: INTEGER);
BEGIN SetFirst(a, n)
END ForwardVar;

PROCEDURE Recursive(a: ARRAY OF INTEGER; n: INTEGER): INTEGER;
VAR result: INTEGER;
BEGIN
  IF n = 0 THEN result := 0 ELSE result := a[n - 1] + Recursive(a, n - 1) END
RETURN result
END Recursive;

PROCEDURE Alias(a: ARRAY OF INTEGER; VAR b: ARRAY OF INTEGER);
BEGIN b[0] := b[0] + 1; Out.Int(a[0], 0); Out.Ln
END Alias;

PROCEDURE PointerSum(a: ARRAY OF Link): INTEGER;
BEGIN RETURN a[0].n + a[1]^.n END PointerSum;

PROCEDURE Mixed(n: INTEGER; p: Link; fixed: Row; a: ARRAY OF INTEGER): INTEGER;
BEGIN RETURN n + p.n + fixed[0] + LEN(a) END Mixed;

PROCEDURE Next(): INTEGER;
BEGIN INC(calls) RETURN 1 END Next;

BEGIN
  one[0] := 7;
  many[0] := 1; many[1] := 2; many[2] := 3; many[3] := 4;
  table[1, 0] := 5; table[1, 1] := 6; table[1, 2] := 7;
  Out.Int(Sum(empty), 0); Out.Ln;
  Out.Int(Sum(one), 0); Out.Ln;
  Out.Int(Forward(many), 0); Out.Ln;
  Out.Int(Recursive(many, LEN(many)), 0); Out.Ln;
  Lengths(one, many);
  ForwardVar(one, 11); Out.Int(one[0], 0); Out.Ln;
  Alias(one, one);
  calls := 0; Out.Int(Sum(table[Next()]), 0); Out.Char(":"); Out.Int(calls, 0); Out.Ln;
  NEW(links[0]); NEW(links[1]); links[0].n := 20; links[1].n := 22;
  Out.Int(PointerSum(links), 0); Out.Ln;
  Out.Int(Mixed(1, links[0], table[1], many), 0); Out.Ln
END OpenArrays.
