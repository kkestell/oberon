MODULE RecordTable;
  IMPORT Out;
  TYPE Table = ARRAY 16 OF RECORD
      ch: CHAR; count: INTEGER; name: ARRAY 6 OF CHAR
    END;
  VAR w, copy: Table;
      k, n: INTEGER;
BEGIN
  k := 3; w[k].ch := "C"; w[k].count := 4; INC(w[k].count);
  w[k].name := "cat";
  w[4] := w[k]; w[4].ch := "D"; w[4].count := 9;
  IF w[k].name = "cat" THEN Out.Int(1, 0) ELSE Out.Int(0, 0) END;
  n := 0;
  WHILE w[k].name[n] # 0X DO INC(n) END;
  Out.Char(w[k].ch); Out.Int(w[k].count, 4); Out.Int(n, 4); Out.Ln;
  Out.Char(w[4].ch); Out.Int(w[4].count, 4); Out.Char(w[4].name[1]); Out.Ln;
  copy := w; w[k].count := 99; w[k].name[0] := "X";
  Out.Int(copy[k].count, 0); Out.Char(copy[k].name[0]);
  Out.Int(copy[4].count, 4); Out.Ln
END RecordTable.
