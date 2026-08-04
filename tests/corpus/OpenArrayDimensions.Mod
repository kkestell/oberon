MODULE OpenArrayDimensions;
IMPORT Out;
TYPE Row = ARRAY 3 OF INTEGER;
VAR grid: ARRAY 2, 3 OF INTEGER;
    cube: ARRAY 2, 3, 4 OF INTEGER;
    rows: ARRAY 2 OF Row;

PROCEDURE Sum(a: ARRAY OF INTEGER): INTEGER;
VAR i, n: INTEGER;
BEGIN i := 0; n := 0; WHILE i < LEN(a) DO n := n + a[i]; INC(i) END
RETURN n
END Sum;

PROCEDURE Inspect2(a: ARRAY OF ARRAY OF INTEGER);
BEGIN
  Out.Int(LEN(a), 0); Out.Char(":"); Out.Int(LEN(a[0]), 0); Out.Char(":");
  Out.Int(a[1, 2], 0); Out.Char(":"); Out.Int(a[1][2], 0); Out.Ln;
  Out.Int(Sum(a[1]), 0); Out.Ln
END Inspect2;

PROCEDURE Inspect3(a: ARRAY OF ARRAY OF ARRAY OF INTEGER);
BEGIN
  Out.Int(LEN(a), 0); Out.Char(":"); Out.Int(LEN(a[0]), 0); Out.Char(":");
  Out.Int(LEN(a[0, 0]), 0); Out.Char(":"); Out.Int(a[1, 2, 3], 0); Out.Ln
END Inspect3;

PROCEDURE Change3(VAR a: ARRAY OF ARRAY OF ARRAY OF INTEGER);
BEGIN a[1][2, 3] := 999
END Change3;

PROCEDURE Mixed(a: ARRAY OF Row; i: INTEGER);
CONST Width = LEN(a[i]);
BEGIN
  Out.Int(Width, 0); Out.Char(":"); Out.Int(LEN(a), 0); Out.Ln;
  Inspect2(a)
END Mixed;

BEGIN
  grid[1, 0] := 10; grid[1, 1] := 20; grid[1, 2] := 30;
  cube[1, 2, 3] := 123;
  rows[1, 0] := 7; rows[1, 1] := 8; rows[1, 2] := 9;
  Inspect2(grid);
  Inspect3(cube);
  Change3(cube); Out.Int(cube[1, 2, 3], 0); Out.Ln;
  Mixed(rows, 27)
END OpenArrayDimensions.
