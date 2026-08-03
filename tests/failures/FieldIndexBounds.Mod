MODULE FieldIndexBounds;
  TYPE Box = RECORD row: ARRAY 3 OF INTEGER END;
  VAR box: Box; i: INTEGER;

  PROCEDURE Read(VAR box: Box; i: INTEGER): INTEGER;
  RETURN box.row[i]
  END Read;

BEGIN
  i := LEN(box.row);
  i := Read(box, i)
END FieldIndexBounds.
