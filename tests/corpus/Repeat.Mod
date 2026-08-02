MODULE Repeat;
  IMPORT Out;
  VAR x: INTEGER;
BEGIN
  x := 7;
  REPEAT
    x := x + 1
  UNTIL x >= 3;
  Out.Int(x, 0); Out.Ln;

  x := 0;
  REPEAT
    x := x + 1
  UNTIL x = 3;
  Out.Int(x, 0); Out.Ln
END Repeat.
