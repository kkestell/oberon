MODULE HailstoneLoop;
IMPORT Out;

PROCEDURE HailStone(start: INTEGER): INTEGER;
VAR n, largest, count: INTEGER;
BEGIN
  n := start; largest := n; count := 1;
  WHILE n # 1 DO
    IF ODD(n) THEN
      n := 3 * n + 1;
      IF n > largest THEN largest := n END
    ELSE
      n := n DIV 2
    END;
    INC(count);
    (* Every trajectory used here takes at most 350 steps. The cap turns
       corruption into a stable mismatch instead of hanging the corpus. *)
    IF count > 2000 THEN n := 1 END
  END
RETURN count
END HailStone;

PROCEDURE FindMax(limit: INTEGER);
VAR start, count, maximum: INTEGER;
BEGIN
  maximum := 0;
  FOR start := 2 TO limit DO
    count := HailStone(start);
    IF count > maximum THEN maximum := count END
  END;
  Out.Int(maximum, 0); Out.Ln
END FindMax;

BEGIN
  FindMax(100000)
END HailstoneLoop.
