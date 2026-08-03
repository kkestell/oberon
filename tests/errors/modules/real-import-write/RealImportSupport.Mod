MODULE RealImportSupport;
  VAR scale*: REAL; count*: INTEGER;

  PROCEDURE Update*(VAR x: REAL);
  BEGIN
    x := x * 2.
  END Update;

BEGIN
  scale := 1.
END RealImportSupport.
