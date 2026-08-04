MODULE NilDerefChain;
TYPE P = POINTER TO R; R = RECORD next: P; n: INTEGER END;
VAR p: P; n: INTEGER;
BEGIN
  NEW(p); p.next := NIL;
  n := p.next.n
END NilDerefChain.
