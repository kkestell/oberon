MODULE NilDerefExplicit;
TYPE P = POINTER TO R; R = RECORD n: INTEGER END;
VAR p: P; n: INTEGER;
BEGIN
  p := NIL;
  n := p^.n
END NilDerefExplicit.
