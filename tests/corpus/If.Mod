MODULE If;
  IMPORT Out;
  VAR x: INTEGER;
BEGIN
  x := 2;
  IF x = 1 THEN
    Out.Int(10, 0)
  ELSIF x = 2 THEN
    IF x # 0 THEN
      Out.Int(20, 0)
    ELSE
      Out.Int(30, 0)
    END
  ELSE
    Out.Int(40, 0)
  END;
  Out.Ln;
  IF x < 0 THEN
    Out.Int(10, 0)
  ELSIF x <= 2 THEN
    Out.Int(21, 0)
  ELSE
    Out.Int(30, 0)
  END;
  Out.Ln
END If.
