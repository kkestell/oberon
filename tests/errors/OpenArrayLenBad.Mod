MODULE OpenArrayLenBad;
TYPE Row = ARRAY 4 OF INTEGER;
VAR grid: ARRAY 2, 3 OF INTEGER;
    rows: ARRAY 2 OF Row;

PROCEDURE Bad(a: ARRAY OF ARRAY OF INTEGER; mixed: ARRAY OF Row; i: INTEGER);
CONST Outer = LEN(a);
      Width = LEN(mixed[i]);
VAR dynamic: ARRAY LEN(a[0]) OF INTEGER;
    fixed: ARRAY Width OF INTEGER;
BEGIN
  i := LEN(a.missing);
  i := a[0, 0, 0];
  i := a[TRUE]
END Bad;
BEGIN Bad(grid, rows, 0)
END OpenArrayLenBad.
