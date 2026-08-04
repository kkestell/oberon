MODULE RecordExtensionLayout;
  IMPORT Out;
  TYPE Link = POINTER TO Root;
       Root = RECORD ch: CHAR; value: INTEGER END;
       Child = RECORD (Root) tag: CHAR; link: Link END;
       Grand = RECORD (Child) count: INTEGER; last: CHAR END;
       Table = ARRAY 3 OF Grand;
  VAR table: Table; p: Link;
BEGIN
  table[0].ch := "A"; table[0].value := 1; table[0].tag := "a"; table[0].count := 10; table[0].last := "x";
  table[1].ch := "B"; table[1].value := 2; table[1].tag := "b"; table[1].count := 20; table[1].last := "y";
  table[2].ch := "C"; table[2].value := 3; table[2].tag := "c"; table[2].count := 30; table[2].last := "z";
  NEW(p); table[1].link := p;
  Out.Char(table[0].ch); Out.Int(table[0].value, 0); Out.Char(table[0].tag); Out.Int(table[0].count, 0); Out.Char(table[0].last); Out.Ln;
  Out.Char(table[1].ch); Out.Int(table[1].value, 0); Out.Char(table[1].tag); Out.Int(table[1].count, 0); Out.Char(table[1].last); Out.Int(ORD(table[1].link # NIL), 0); Out.Ln;
  Out.Char(table[2].ch); Out.Int(table[2].value, 0); Out.Char(table[2].tag); Out.Int(table[2].count, 0); Out.Char(table[2].last); Out.Ln
END RecordExtensionLayout.
