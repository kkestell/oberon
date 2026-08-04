MODULE PointerBad;
TYPE
  R = RECORD n: INTEGER END;
  S = RECORD n: INTEGER END;
  P = POINTER TO R;
  Q = POINTER TO S;
  BadInt = POINTER TO INTEGER;
  BadArray = POINTER TO ARRAY 2 OF INTEGER;
  BadPointer = POINTER TO P;
VAR p: P; q: Q; i: INTEGER; b: BOOLEAN;
BEGIN
  i := i^;
  i := p.missing;
  i := p + p;
  b := p < p;
  IF p THEN i := 0 END;
  CASE p OF 0: i := 0 END;
  p := q;
  b := p = q;
  i := NIL;
  i := NIL + NIL
END PointerBad.
