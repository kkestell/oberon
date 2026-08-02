MODULE SiblingRef;
PROCEDURE Outer;
  PROCEDURE First;
  BEGIN
    Second
  END First;
  PROCEDURE Second;
  END Second;
BEGIN
  First
END Outer;
BEGIN
  Outer
END SiblingRef.
