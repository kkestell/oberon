MODULE OpenArrayImportWrite;
IMPORT OpenArraySource;
PROCEDURE Read(a: ARRAY OF ARRAY OF INTEGER); END Read;
PROCEDURE ReadRow(a: ARRAY OF INTEGER); END ReadRow;
PROCEDURE Write(VAR a: ARRAY OF ARRAY OF INTEGER); END Write;
PROCEDURE WriteRow(VAR a: ARRAY OF INTEGER); END WriteRow;
BEGIN
  Read(OpenArraySource.data);
  ReadRow(OpenArraySource.data[0]);
  Write(OpenArraySource.data);
  WriteRow(OpenArraySource.data[0])
END OpenArrayImportWrite.
