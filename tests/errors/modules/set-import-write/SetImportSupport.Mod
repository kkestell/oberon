MODULE SetImportSupport;
  VAR flags*: SET;

  PROCEDURE Update*(VAR s: SET);
  BEGIN
    INCL(s, 1)
  END Update;

BEGIN
  flags := {0}
END SetImportSupport.
