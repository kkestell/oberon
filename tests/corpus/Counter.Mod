MODULE Counter;
IMPORT Out;
VAR count: INTEGER;

PROCEDURE Bump(n: INTEGER);
BEGIN
  count := count + n
END Bump;

PROCEDURE Read(): INTEGER;
RETURN count
END Read;

BEGIN
  count := 40; Bump(1); Bump(1);
  Out.Int(Read(), 0); Out.Ln
END Counter.
