MODULE Support;
  VAR count*: INTEGER;

  PROCEDURE Twice*(VAR n: INTEGER);
  BEGIN
    n := n * 2
  END Twice;

BEGIN
  count := 1
END Support.
