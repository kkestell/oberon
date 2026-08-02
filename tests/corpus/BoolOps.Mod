MODULE BoolOps;
  IMPORT Out;
  CONST
    Always = TRUE;
    Never = FALSE;
    Same = TRUE = ~FALSE;
    Different = Always # Never;
    Folded = (FALSE & (1 DIV 0 = 0)) OR TRUE;
  VAR a, b: BOOLEAN;
BEGIN
  a := Always;
  b := ~a OR Never;
  IF Same & Different & Folded & ~b THEN
    Out.Int(1, 0)
  ELSE
    Out.Int(0, 0)
  END;
  IF a = (b # TRUE) THEN
    Out.Int(2, 0)
  ELSE
    Out.Int(0, 0)
  END;
  Out.Ln
END BoolOps.
