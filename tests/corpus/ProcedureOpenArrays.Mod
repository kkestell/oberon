MODULE ProcedureOpenArrays;
IMPORT Out;
TYPE Base = RECORD n: INTEGER END;
  Derived = RECORD (Base) extra: INTEGER END;
  OpenCallback = PROCEDURE(a: ARRAY OF ARRAY OF INTEGER);
  RecordCallback = PROCEDURE(VAR r: Base);
  Grid = ARRAY 2, 3 OF INTEGER;
VAR open: OpenCallback;
  record: RecordCallback;
  grid: Grid;
  item: Derived;

PROCEDURE Shape(a: ARRAY OF ARRAY OF INTEGER);
BEGIN Out.Int(LEN(a), 1); Out.Char("x"); Out.Int(LEN(a[0]), 1); Out.Char(" ")
END Shape;

PROCEDURE Kind(VAR r: Base);
BEGIN
  IF r IS Derived THEN Out.Char("D") ELSE Out.Char("B") END
END Kind;

BEGIN
  Shape(grid);
  open := Shape; open(grid);
  Kind(item);
  record := Kind; record(item); Out.Ln
END ProcedureOpenArrays.
