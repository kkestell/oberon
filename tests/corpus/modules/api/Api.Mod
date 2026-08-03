MODULE Api;
  (* Two qualifiers for one module. The graph holds one ApiSupport, so both
     aliases name its data and its procedures. *)
  IMPORT Out, A := ApiSupport, B := ApiSupport;
  CONST Doubled = A.Limit * 2;
  VAR half, rest: INTEGER;
BEGIN
  Out.Int(A.Limit, 0); Out.Int(Doubled, 4); Out.Ln;
  IF A.Ready & B.open THEN Out.Int(1, 0) ELSE Out.Int(0, 0) END;
  Out.Ln;
  Out.Int(A.count, 0); Out.Int(B.count, 4); Out.Ln;
  A.Bump(4);
  Out.Int(A.count, 0); Out.Int(B.count, 4); Out.Ln;
  Out.Int(B.Twice(A.count), 0); Out.Ln;
  B.Split(A.count + 2, half, rest);
  Out.Int(half, 0); Out.Int(rest, 4); Out.Ln
END Api.
