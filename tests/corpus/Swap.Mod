MODULE Swap;
IMPORT Out;
VAR a, b: INTEGER; state: BOOLEAN;

PROCEDURE Swap(VAR x, y: INTEGER; VAR flag: BOOLEAN);
VAR temp: INTEGER;
BEGIN
  temp := x; x := y; y := temp; flag := ~flag
END Swap;

PROCEDURE Local;
VAR x, y: INTEGER; flag: BOOLEAN;
BEGIN
  x := 4; y := 9; flag := FALSE;
  Swap(x, y, flag);
  Out.Int(x, 0); Out.Int(y, 2);
  IF flag THEN Out.Int(1, 2) ELSE Out.Int(0, 2) END;
  Out.Ln
END Local;

BEGIN
  a := 1; b := 2; state := TRUE;
  Swap(a, b, state);
  Out.Int(a, 0); Out.Int(b, 2);
  IF state THEN Out.Int(1, 2) ELSE Out.Int(0, 2) END;
  Out.Ln;
  Local
END Swap.
