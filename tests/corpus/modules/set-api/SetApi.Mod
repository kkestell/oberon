MODULE SetApi;
  (* The client folds the imported SET constant, reads the imported variable
     the support module's body initialized, and calls both exported
     procedure forms. *)
  IMPORT Out, S := SetSupport;
  CONST Widened = S.Mask + {0};
  VAR local: SET;
BEGIN
  Out.Int(ORD(S.Mask), 14); Out.Int(ORD(Widened), 14); Out.Ln;
  Out.Int(ORD(S.flags), 14); Out.Ln;
  local := S.flags;
  S.Merge(S.Mask, local);
  Out.Int(ORD(local), 14); Out.Ln;
  Out.Int(ORD(S.Range(2, 4)), 14); Out.Int(ORD(S.Range(4, 2)), 14); Out.Ln
END SetApi.
