MODULE ImportedWrite;
  IMPORT Support;
BEGIN
  Support.count := 1;
  Support.Twice(Support.count);
  INC(Support.count);
  DEC(Support.count)
END ImportedWrite.
