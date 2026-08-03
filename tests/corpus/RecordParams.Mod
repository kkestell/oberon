MODULE RecordParams;
  IMPORT Out;
  TYPE
    Pair = RECORD left, right: INTEGER; row: ARRAY 3 OF INTEGER END;
    Row = ARRAY 3 OF INTEGER;
  VAR p: Pair; a: Row;

  PROCEDURE SumPair(p: Pair): INTEGER;
    CONST Count = LEN(p.row);
  RETURN p.left + p.right + p.row[0] + p.row[1] + p.row[2] + Count - 3
  END SumPair;

  PROCEDURE SumRow(a: Row): INTEGER;
  RETURN a[0] + a[1] + a[2]
  END SumRow;

  PROCEDURE Change(VAR p: Pair; VAR a: Row);
  BEGIN p.left := p.left + 10; p.row[2] := 30; a[1] := 20 END Change;

  PROCEDURE Recur(VAR p: Pair; n: INTEGER);
  BEGIN
    IF n > 0 THEN INC(p.right); Recur(p, n - 1) END
  END Recur;

  PROCEDURE Forward(p: Pair; a: Row): INTEGER;
  RETURN SumPair(p) + SumRow(a)
  END Forward;

BEGIN
  p.left := 1; p.right := 2; p.row[0] := 3; p.row[1] := 4; p.row[2] := 5;
  a[0] := 6; a[1] := 7; a[2] := 8;
  Out.Int(Forward(p, a), 0); Out.Ln;
  Change(p, a); Recur(p, 3);
  Out.Int(SumPair(p), 0); Out.Int(SumRow(a), 4); Out.Ln
END RecordParams.
