MODULE RecordImportWrite;
  IMPORT RecordWriteSource;
  VAR local: RecordWriteSource.Box;

  PROCEDURE Change(VAR box: RecordWriteSource.Box);
  END Change;

BEGIN
  RecordWriteSource.box := local;
  RecordWriteSource.box.n := 1;
  RecordWriteSource.box.row[0] := 2;
  INC(RecordWriteSource.box.n);
  Change(RecordWriteSource.box)
END RecordImportWrite.
