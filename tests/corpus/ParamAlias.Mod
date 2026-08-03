MODULE ParamAlias;
  IMPORT Out;
  TYPE Box = RECORD n: INTEGER END;
       Row = ARRAY 2 OF INTEGER;
  VAR box: Box; row: Row;

  PROCEDURE RecordAlias(value: Box; VAR changed: Box);
  BEGIN
    Out.Int(value.n, 0); changed.n := 41; Out.Int(value.n, 4); Out.Ln
  END RecordAlias;

  PROCEDURE ArrayAlias(value: Row; VAR changed: Row);
  BEGIN
    Out.Int(value[0], 0); changed[0] := 42; Out.Int(value[0], 4); Out.Ln
  END ArrayAlias;

BEGIN
  box.n := 4; row[0] := 5;
  RecordAlias(box, box); ArrayAlias(row, row)
END ParamAlias.
