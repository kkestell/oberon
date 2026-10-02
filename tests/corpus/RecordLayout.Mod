MODULE RecordLayout;
  IMPORT Out;
  TYPE
    Padded = RECORD ch: CHAR; count: INTEGER END;
    Mixed = RECORD first: REAL; bytes: ARRAY 3 OF BYTE; last: REAL END;
    (* A four-byte BOOLEAN and a CHAR between eight-byte fields, so every
       field but the first sits after padding. *)
    Scalars = RECORD flag: BOOLEAN; ch: CHAR; n: INTEGER; done: BOOLEAN; r: REAL END;
  VAR a, b: Padded;
      x, y: Mixed;
      p, q: ARRAY 3 OF Scalars; i: INTEGER;

  PROCEDURE Local;
    VAR ch: CHAR; value: Padded; n: INTEGER;
  BEGIN
    ch := "L"; value.ch := "P"; value.count := 31; n := 32;
    Out.Char(ch); Out.Char(value.ch); Out.Int(value.count, 4); Out.Int(n, 4); Out.Ln
  END Local;
BEGIN
  a.ch := "A"; a.count := 123456; b := a;
  x.first := 1.25; x.bytes[0] := 7; x.bytes[1] := 8; x.bytes[2] := 9;
  x.last := -2.5; y := x;
  a.ch := "Z"; a.count := 0; x.first := 0.0; x.bytes[1] := 0; x.last := 0.0;
  Out.Char(b.ch); Out.Int(b.count, 8); Out.Ln;
  Out.Int(FLOOR(y.first * 100.0), 0); Out.Int(y.bytes[0], 4); Out.Int(y.bytes[1], 4);
  Out.Int(y.bytes[2], 4); Out.Int(FLOOR(y.last * 100.0), 6); Out.Ln;
  FOR i := 0 TO 2 DO
    p[i].flag := ODD(i); p[i].ch := CHR(ORD("a") + i); p[i].n := -9223372036854775807 + i;
    p[i].done := ~ODD(i); p[i].r := FLT(i) + 0.5
  END;
  q := p;
  FOR i := 0 TO 2 DO p[i].flag := FALSE; p[i].ch := "-"; p[i].n := 0; p[i].done := FALSE; p[i].r := 0.0 END;
  FOR i := 0 TO 2 DO
    Out.Int(ORD(q[i].flag), 2); Out.Char(" "); Out.Char(q[i].ch); Out.Int(q[i].n, 21);
    Out.Int(ORD(q[i].done), 2); Out.Int(FLOOR(q[i].r * 10.0), 4); Out.Ln
  END;
  Local
END RecordLayout.
