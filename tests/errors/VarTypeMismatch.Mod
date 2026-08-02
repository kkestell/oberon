MODULE VarTypeMismatch;
VAR flag: BOOLEAN;
PROCEDURE Set(VAR x: INTEGER);
BEGIN
  x := 1
END Set;
BEGIN
  Set(flag)
END VarTypeMismatch.
