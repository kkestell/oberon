MODULE While;
  IMPORT Out;
  VAR i, sum, x, y: INTEGER;
BEGIN
  i := 0; sum := 0;
  WHILE i < 5 DO
    sum := sum + i;
    i := i + 1
  END;
  Out.Int(sum, 0); Out.Ln;

  x := 48; y := 18;
  WHILE x > y DO
    x := x - y
  ELSIF y > x DO
    y := y - x
  END;
  Out.Int(x, 0); Out.Ln
END While.
