MODULE VarActualExpr;
CONST C = 1;
PROCEDURE Set(VAR x, y: INTEGER);
BEGIN
  x := y
END Set;
BEGIN
  Set(C, C + 1)
END VarActualExpr.
