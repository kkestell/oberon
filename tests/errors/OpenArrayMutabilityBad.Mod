MODULE OpenArrayMutabilityBad;
TYPE R = RECORD n: INTEGER END;
     P = POINTER TO R;
VAR records: ARRAY 2 OF R;
    pointers: ARRAY 2 OF P;
    grid: ARRAY 2, 2 OF INTEGER;

PROCEDURE Change(VAR a: ARRAY OF INTEGER); END Change;
PROCEDURE ChangePointer(VAR p: P); END ChangePointer;

PROCEDURE Bad(a: ARRAY OF R; p: ARRAY OF P; nested: ARRAY OF ARRAY OF INTEGER);
BEGIN
  a := a;
  a[0].n := 1;
  nested[0, 0] := 2;
  Change(nested[0]);
  INC(nested[0, 0]);
  p[0].n := 3;
  p[0]^.n := 4;
  NEW(p[0]);
  ChangePointer(p[0])
END Bad;
BEGIN Bad(records, pointers, grid)
END OpenArrayMutabilityBad.
