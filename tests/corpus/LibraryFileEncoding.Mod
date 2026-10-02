MODULE LibraryFileEncoding;

IMPORT Files, Out;

VAR f: Files.File; r: Files.Rider; data: ARRAY 45 OF BYTE; i: INTEGER;

BEGIN
  f := Files.New("encoding.bin"); ASSERT(f # NIL); Files.Set(r, f, 0);
  Files.WriteInt(r, 01020304H); Files.WriteReal(r, 1.0); Files.WriteSet(r, {0, 63}); Files.WriteBool(r, TRUE);
  Files.WriteNum(r, -9223372036854775808); Files.WriteNum(r, 9223372036854775807); ASSERT(Files.Pos(r) = 45);
  Files.Set(r, f, 0); Files.ReadBytes(r, data, 45); ASSERT(r.res = 0);
  ASSERT((data[0] = 4) & (data[1] = 3) & (data[2] = 2) & (data[3] = 1));
  i := 4; WHILE i < 14 DO ASSERT(data[i] = 0); INC(i) END; ASSERT((data[14] = 240) & (data[15] = 63));
  ASSERT(data[16] = 1); i := 17; WHILE i < 23 DO ASSERT(data[i] = 0); INC(i) END; ASSERT(data[23] = 128);
  ASSERT(data[24] = 1);
  i := 25; WHILE i < 34 DO ASSERT(data[i] = 128); INC(i) END; ASSERT(data[34] = 127);
  i := 35; WHILE i < 44 DO ASSERT(data[i] = 255); INC(i) END; ASSERT(data[44] = 0);
  Out.String("ok"); Out.Ln
END LibraryFileEncoding.
