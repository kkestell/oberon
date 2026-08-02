MODULE Arith;
  IMPORT Out;
  CONST N = 7;
  VAR x, y: INTEGER;
BEGIN
  x := N * 6;
  y := x DIV 2 + 1;
  Out.Int(y, 0); Out.Ln
END Arith.
