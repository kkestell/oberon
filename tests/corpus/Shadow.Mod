MODULE Shadow;
IMPORT Out;
VAR x: INTEGER;

PROCEDURE P;
VAR x: INTEGER;
  PROCEDURE Q;
  VAR x: INTEGER;
  BEGIN
    x := 3;
    Out.Int(x, 3)
  END Q;
  PROCEDURE P;
  BEGIN
    Out.Int(4, 3)
  END P;
  PROCEDURE Param(x: INTEGER);
  BEGIN
    Out.Int(x, 3)
  END Param;
BEGIN
  x := 2;
  Out.Int(x, 0);
  Q;
  P;
  Param(5)
END P;

BEGIN
  x := 1;
  P;
  Out.Int(x, 3); Out.Ln
END Shadow.
