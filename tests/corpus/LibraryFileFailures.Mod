MODULE LibraryFileFailures;

IMPORT Files, Out;

VAR f: Files.File; r: Files.Rider; data: ARRAY 6 OF BYTE; tiny: ARRAY 2 OF BYTE; small: ARRAY 3 OF CHAR; message: ARRAY 5 OF CHAR;
    i: INTEGER; flag: BOOLEAN; res: INTEGER;

BEGIN
  f := Files.New("failures.bin"); ASSERT(f # NIL); Files.Set(r, f, 0);
  data[0] := 1; data[1] := 2; Files.WriteBytes(r, data, 2); Files.Set(r, f, 0);
  i := 99; Files.ReadInt(r, i); ASSERT((i = 99) & r.eof & (r.res = 2) & (Files.Pos(r) = 2));
  Files.GetError(message); ASSERT(message # "");

  Files.Purge(f); Files.Set(r, f, 0); data[0] := 1; data[1] := 2; data[2] := 3; data[3] := 4;
  Files.WriteBytes(r, data, 4); Files.Set(r, f, 0); Files.ReadBytes(r, tiny, 4);
  ASSERT(~r.eof & (r.res = 2) & (Files.Pos(r) = 2) & (tiny[0] = 1) & (tiny[1] = 2));

  Files.Purge(f); Files.Set(r, f, 0); Files.Write(r, 2); Files.Set(r, f, 0);
  flag := TRUE; Files.ReadBool(r, flag); ASSERT(flag & ~r.eof & (r.res = 1) & (Files.Pos(r) = 1));

  Files.Purge(f); Files.Set(r, f, 0);
  data[0] := 128; data[1] := 128; data[2] := 128; data[3] := 128; data[4] := 128; data[5] := 0;
  Files.WriteBytes(r, data, 6); Files.Set(r, f, 0);
  i := 77; Files.ReadNum(r, i); ASSERT((i = 77) & ~r.eof & (r.res = 1) & (Files.Pos(r) = 6));

  Files.Purge(f); Files.Set(r, f, 0); Files.WriteString(r, "hello"); Files.Set(r, f, 0);
  small := "xx"; Files.ReadString(r, small); ASSERT((small = "") & ~r.eof & (r.res = 1) & (Files.Pos(r) = 6));

  Files.Purge(f); Files.Set(r, f, 0); data[0] := 97; data[1] := 98; data[2] := 99;
  Files.WriteBytes(r, data, 3); Files.Set(r, f, 0);
  small := "xx"; Files.ReadString(r, small); ASSERT((small = "") & r.eof & (r.res = 1) & (Files.Pos(r) = 3));

  Files.Delete("absent.bin", res); ASSERT(res # 0); Files.GetError(message); ASSERT(message[4] = 0X);
  Out.String("ok"); Out.Ln
END LibraryFileFailures.
