MODULE RecordLayout;
  IMPORT Out;
  TYPE
    Padded = RECORD ch: CHAR; count: INTEGER END;
    Mixed = RECORD first: REAL; bytes: ARRAY 3 OF BYTE; last: REAL END;
  VAR a, b: Padded;
      x, y: Mixed;

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
  Local
END RecordLayout.
