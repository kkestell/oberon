MODULE CharApi;
  (* The client. The imported string constant reaches this module as its
     bytes, so the assignment and the comparisons below emit this module's
     own literal data and no symbol crosses the boundary. *)
  IMPORT Out, CharTypes;
  VAR mine: CharTypes.Name;
      i: INTEGER;
      ch: CHAR;
BEGIN
  mine := CharTypes.Greeting;
  Out.Int(ORD(mine = CharTypes.Greeting), 4);
  Out.Int(ORD(mine # CharTypes.label), 4);
  Out.Int(ORD(mine > CharTypes.label), 4);
  (* the imported CHAR constant in a relation *)
  Out.Int(ORD(CharTypes.Mark < CharTypes.initial), 4);
  Out.Ln;
  (* the imported BYTE variable reads into an INTEGER *)
  i := CharTypes.flags;
  Out.Int(i, 4); Out.Ln;
  (* the imported CHAR constant as a case label *)
  ch := CharTypes.initial;
  CASE ch OF
    CharTypes.Mark: Out.Char("!")
  | "A" .. "Z": Out.Char("u")
  END;
  Out.Ln
END CharApi.
