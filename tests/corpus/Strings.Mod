MODULE Strings;
  (* Report 9.1 exception 2: a string assigns to a character array by copying
     its characters and one appended null, and nothing else. The scans prove
     where the terminator sits, and the sentinel proves the copy left the
     rest of the destination untouched. *)
  IMPORT Out;
  CONST Greet = "hello"; Empty = ""; Nul = 0X;
  VAR exact: ARRAY 6 OF CHAR;
      wide: ARRAY 10 OF CHAR;
      pair: ARRAY 2 OF CHAR;
      other: ARRAY 6 OF CHAR;
      ch: CHAR;
      i: INTEGER;
BEGIN
  (* the exact fit: five characters and the terminator fill all six elements *)
  exact := Greet;
  i := 0;
  WHILE exact[i] # 0X DO Out.Char(exact[i]); INC(i) END;
  Out.Ln;
  Out.Int(i, 4); Out.Ln;
  (* the terminator lands right after the characters, not at the end *)
  wide := "hi";
  i := 0;
  WHILE wide[i] # Nul DO INC(i) END;
  Out.Int(i, 4); Out.Ln;
  (* the copy is the character count plus one: a sentinel beyond survives *)
  wide[9] := "x";
  wide := "hey";
  Out.Char(wide[9]); Out.Ln;
  (* the empty string is one null byte into any array of length one or more *)
  pair[0] := "q"; pair[1] := "q";
  pair := Empty;
  Out.Int(ORD(pair[0]), 4); Out.Int(ORD(pair[1]), 4); Out.Ln;
  (* single-character quoted and ordinal literals, to a CHAR and to a
     two-element array *)
  ch := "A"; Out.Char(ch);
  ch := 2AX; Out.Char(ch);
  pair := "B"; Out.Char(pair[0]); Out.Int(ORD(pair[1]), 4);
  pair := 2BX; Out.Char(pair[0]); Out.Int(ORD(pair[1]), 4); Out.Ln;
  (* assignment is a copy, not an alias: mutating one destination leaves a
     second one assigned from the same literal untouched *)
  exact := "abc"; other := "abc";
  exact[0] := "x";
  i := 0;
  WHILE exact[i] # 0X DO Out.Char(exact[i]); INC(i) END;
  Out.Char(" ");
  i := 0;
  WHILE other[i] # 0X DO Out.Char(other[i]); INC(i) END;
  Out.Ln
END Strings.
