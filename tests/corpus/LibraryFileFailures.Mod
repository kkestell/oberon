MODULE LibraryFileFailures;

IMPORT Files, Out;

VAR f: Files.File; r: Files.Rider; data: ARRAY 11 OF BYTE; tiny: ARRAY 2 OF BYTE; small: ARRAY 3 OF CHAR; message: ARRAY 5 OF CHAR;
    i: INTEGER; flag: BOOLEAN; res: INTEGER;

BEGIN
  f := Files.New("failures.bin"); ASSERT(f # NIL); Files.Set(r, f, 0);
  data[0] := 1; data[1] := 2; Files.WriteBytes(r, data, 2); Files.Set(r, f, 0);
  i := 99; Files.ReadInt(r, i); ASSERT((i = 99) & r.eof & (r.res = 6) & (Files.Pos(r) = 2));
  Files.GetError(message); ASSERT(message # "");

  Files.Purge(f); Files.Set(r, f, 0); data[0] := 1; data[1] := 2; data[2] := 3; data[3] := 4;
  Files.WriteBytes(r, data, 4); Files.Set(r, f, 0); Files.ReadBytes(r, tiny, 4);
  ASSERT(~r.eof & (r.res = 2) & (Files.Pos(r) = 2) & (tiny[0] = 1) & (tiny[1] = 2));

  Files.Purge(f); Files.Set(r, f, 0); Files.Write(r, 2); Files.Set(r, f, 0);
  flag := TRUE; Files.ReadBool(r, flag); ASSERT(flag & ~r.eof & (r.res = 1) & (Files.Pos(r) = 1));

  (* Ten continuation bytes are longer than any INTEGER encoding, and after
     nine the final byte can only be a sign. *)
  Files.Purge(f); Files.Set(r, f, 0);
  i := 0; WHILE i < 10 DO data[i] := 128; INC(i) END; data[10] := 0;
  Files.WriteBytes(r, data, 11); Files.Set(r, f, 0);
  i := 77; Files.ReadNum(r, i); ASSERT((i = 77) & ~r.eof & (r.res = 1) & (Files.Pos(r) = 11));
  Files.Purge(f); Files.Set(r, f, 0); data[9] := 1;
  Files.WriteBytes(r, data, 10); Files.Set(r, f, 0);
  i := 77; Files.ReadNum(r, i); ASSERT((i = 77) & ~r.eof & (r.res = 1) & (Files.Pos(r) = 10));

  Files.Purge(f); Files.Set(r, f, 0); Files.WriteString(r, "hello"); Files.Set(r, f, 0);
  small := "xx"; Files.ReadString(r, small); ASSERT((small = "") & ~r.eof & (r.res = 1) & (Files.Pos(r) = 6));

  Files.Purge(f); Files.Set(r, f, 0); data[0] := 97; data[1] := 98; data[2] := 99;
  Files.WriteBytes(r, data, 3); Files.Set(r, f, 0);
  small := "xx"; Files.ReadString(r, small); ASSERT((small = "") & r.eof & (r.res = 1) & (Files.Pos(r) = 3));

  Files.Delete("absent.bin", res); ASSERT(res # 0); Files.GetError(message); ASSERT(message[4] = 0X);
  Out.String("ok"); Out.Ln
END LibraryFileFailures.
