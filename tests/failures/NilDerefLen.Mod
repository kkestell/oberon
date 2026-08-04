MODULE NilDerefLen;
TYPE P = POINTER TO R; R = RECORD row: ARRAY 3 OF INTEGER END;
VAR p: P; n: INTEGER;
BEGIN
  p := NIL;
  n := LEN(p.row)
END NilDerefLen.
