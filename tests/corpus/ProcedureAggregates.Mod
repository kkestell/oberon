MODULE ProcedureAggregates;
IMPORT Out;
TYPE Action = PROCEDURE;
  Actions = ARRAY 3 OF Action;
  Pair = RECORD first, second: Action END;
  PairPointer = POINTER TO Pair;
  Base = RECORD action: Action END;
  Extended = RECORD (Base) marker: INTEGER END;
VAR a, b: Actions;
  r, s: Pair;
  p: PairPointer;
  extended: Extended;

PROCEDURE A;
BEGIN Out.Char("A")
END A;

PROCEDURE B;
BEGIN Out.Char("B")
END B;

PROCEDURE C;
BEGIN Out.Char("C")
END C;

PROCEDURE Invoke(VAR base: Base);
BEGIN base(Extended).action()
END Invoke;

BEGIN
  a[0] := A; a[1] := B; a[2] := C;
  b := a;
  b[0]; b[1](); b[2];
  r.first := C; r.second := A;
  s := r;
  s.first(); s.second;
  NEW(p); p.first := B; p.second := C;
  p^.first; p.second();
  extended.action := A; Invoke(extended); Out.Ln
END ProcedureAggregates.
