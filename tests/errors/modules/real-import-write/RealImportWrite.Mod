MODULE RealImportWrite;
  (* Report 9.1 and 11: an imported variable is read-only in the client, so
     PACK and UNPK cannot reach it any more than assignment or a VAR
     parameter can. Both of the positions UNPK changes are covered. *)
  IMPORT RealImportSupport;
  VAR x: REAL; n: INTEGER;
BEGIN
  RealImportSupport.scale := 2.;
  RealImportSupport.Update(RealImportSupport.scale);
  PACK(RealImportSupport.scale, 1);
  UNPK(RealImportSupport.scale, n);
  UNPK(x, RealImportSupport.count)
END RealImportWrite.
