MODULE PointerImportWrite;
IMPORT PointerSource;
VAR local: PointerSource.P;
PROCEDURE Change(VAR p: PointerSource.P);
BEGIN
END Change;
BEGIN
  PointerSource.p := NIL;
  Change(PointerSource.p);
  NEW(PointerSource.p);
  PointerSource.p.n := 1;
  PointerSource.holder.p.n := 2;
  local := NIL
END PointerImportWrite.
