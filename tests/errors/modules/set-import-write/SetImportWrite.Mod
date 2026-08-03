MODULE SetImportWrite;
  (* Report 9.1 and 11: an imported variable is read-only in the client, so
     INCL and EXCL cannot reach it any more than assignment can. *)
  IMPORT SetImportSupport;
BEGIN
  INCL(SetImportSupport.flags, 1);
  EXCL(SetImportSupport.flags, 0);
  SetImportSupport.Update(SetImportSupport.flags)
END SetImportWrite.
