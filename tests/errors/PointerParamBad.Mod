MODULE PointerParamBad;
TYPE
  R = RECORD n: INTEGER END;
  S = RECORD n: INTEGER END;
  P = POINTER TO R;
  P2 = POINTER TO R;
  Q = POINTER TO S;
VAR p: P; p2: P2; q: Q; i: INTEGER;

PROCEDURE Change(VAR value: P);
BEGIN
END Change;

PROCEDURE Take(value: P);
BEGIN
END Take;

PROCEDURE BadResult(): P;
BEGIN
RETURN q
END BadResult;

PROCEDURE IntegerResult(): P;
BEGIN
RETURN i
END IntegerResult;

BEGIN
  NEW;
  NEW(p, p);
  NEW(NIL);
  NEW(i);
  Change(NIL);
  Change(p2);
  Take(q)
END PointerParamBad.
