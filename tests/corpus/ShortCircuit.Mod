MODULE ShortCircuit;
  IMPORT Out;
  VAR x, y: INTEGER;
BEGIN
  x := 7; y := 0;
  IF (y # 0) & (x DIV y = 0) THEN
    Out.Int(0, 0)
  ELSE
    Out.Int(1, 0)
  END;
  IF (y = 0) OR (x DIV y = 0) THEN
    Out.Int(2, 0)
  ELSE
    Out.Int(0, 0)
  END;
  Out.Ln
END ShortCircuit.
