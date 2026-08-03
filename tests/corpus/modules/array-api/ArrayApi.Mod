MODULE ArrayApi;
  (* A client of both. An imported array is read-only, so it is copied into a
     local one before being changed, and the copy proves it did not alias. *)
  IMPORT Out, ArrayTypes, ArrayFacade;
  VAR mine: ArrayTypes.Row;
      viaFacade: ArrayFacade.Row;
      i: INTEGER;
BEGIN
  mine := ArrayTypes.shared;
  (* Both names denote the one type, so this assignment is between identical
     types and would not compile if the identity had been rebuilt. *)
  viaFacade := mine;
  mine[0] := 99;
  Out.Int(ArrayTypes.shared[0], 0); Out.Int(mine[0], 4);
  Out.Int(viaFacade[0], 4); Out.Ln;
  FOR i := 0 TO LEN(ArrayTypes.shared) - 1 DO
    Out.Int(ArrayTypes.shared[i], 4)
  END;
  Out.Ln;
  (* Exported variables whose types the client cannot name are still readable
     and indexable, and LEN still answers for them. *)
  Out.Int(LEN(ArrayTypes.shared), 4); Out.Int(LEN(ArrayTypes.secret), 4);
  Out.Int(LEN(ArrayTypes.inline), 4); Out.Ln;
  Out.Int(ArrayTypes.secret[1], 4); Out.Int(ArrayTypes.inline[3], 4); Out.Ln
END ArrayApi.
