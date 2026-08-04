MODULE PointerLenBad;
TYPE P = POINTER TO R; R = RECORD row: ARRAY 2 OF ARRAY 3 OF INTEGER END;
VAR p: P; a: ARRAY 2 OF ARRAY 3 OF INTEGER; b: BOOLEAN;
PROCEDURE Index(): INTEGER;
BEGIN
RETURN 0
END Index;
PROCEDURE Test;
CONST A = LEN(p.missing); B = LEN(p^.row[b]); C = LEN(p.row[2]);
  D = LEN(p.row[Index()]); E = LEN(a[Index() + 1]);
BEGIN
END Test;
PROCEDURE Length;
VAR v: ARRAY LEN(a[Index()]) OF INTEGER;
BEGIN
END Length;
END PointerLenBad.
