MODULE CharTable;
  (* An array of character arrays: a string assigns to a selected row, rows
     and characters index through both bracket spellings, two rows compare as
     character arrays, and LEN answers for the table and for a row. *)
  IMPORT Out;
  VAR table: ARRAY 3, 6 OF CHAR;
      i, j: INTEGER;
BEGIN
  table[0] := "alpha";
  table[1] := "beta";
  table[2] := "beta";
  FOR i := 0 TO LEN(table) - 1 DO
    j := 0;
    WHILE table[i][j] # 0X DO Out.Char(table[i, j]); INC(j) END;
    Out.Ln
  END;
  Out.Int(LEN(table), 4); Out.Int(LEN(table[0]), 4); Out.Ln;
  Out.Int(ORD(table[1] = table[2]), 4);
  Out.Int(ORD(table[0] < table[1]), 4);
  Out.Int(ORD(table[0] = "alpha"), 4);
  Out.Int(ORD("beta" # table[2]), 4);
  Out.Ln
END CharTable.
