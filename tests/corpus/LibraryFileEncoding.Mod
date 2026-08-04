MODULE LibraryFileEncoding;

IMPORT Files, Out;

VAR f: Files.File; r: Files.Rider; data: ARRAY 23 OF BYTE; i: INTEGER;

BEGIN
  f := Files.New("encoding.bin"); ASSERT(f # NIL); Files.Set(r, f, 0);
  Files.WriteInt(r, 01020304H); Files.WriteReal(r, 1.0); Files.WriteSet(r, {0, 31}); Files.WriteBool(r, TRUE);
  Files.WriteNum(r, -2147483647 - 1); Files.WriteNum(r, 2147483647); ASSERT(Files.Pos(r) = 23);
  Files.Set(r, f, 0); Files.ReadBytes(r, data, 23); ASSERT(r.res = 0);
  ASSERT((data[0] = 4) & (data[1] = 3) & (data[2] = 2) & (data[3] = 1));
  ASSERT((data[4] = 0) & (data[5] = 0) & (data[6] = 128) & (data[7] = 63));
  ASSERT((data[8] = 1) & (data[9] = 0) & (data[10] = 0) & (data[11] = 128) & (data[12] = 1));
  i := 13; WHILE i < 17 DO ASSERT(data[i] = 128); INC(i) END; ASSERT(data[17] = 120);
  i := 18; WHILE i < 22 DO ASSERT(data[i] = 255); INC(i) END; ASSERT(data[22] = 7);
  Out.String("ok"); Out.Ln
END LibraryFileEncoding.
