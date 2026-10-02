MODULE LibraryFiles;

IMPORT Files, Out;

TYPE OpenFile = PROCEDURE(name: ARRAY OF CHAR): Files.File;
  ExtendedRider = RECORD (Files.Rider) marker: INTEGER END;

VAR f, old: Files.File; r: Files.Rider; b: BYTE; i, t, d, res: INTEGER; x: REAL; set: SET; flag: BOOLEAN;
    text: ARRAY 16 OF CHAR; error: ARRAY 8 OF CHAR; bytes: ARRAY 4 OF BYTE; open: OpenFile; extended: ExtendedRider;

BEGIN
  Files.GetError(error); ASSERT(error = "");
  open := Files.Old; ASSERT(open("missing.bin") = NIL); Files.GetError(error); ASSERT(error # "");
  f := Files.New("values.bin"); ASSERT(f # NIL);
  Files.GetError(error); ASSERT(error # "");
  Files.Set(r, f, 0); ASSERT((Files.Pos(r) = 0) & (Files.Base(r) = f) & ~r.eof & (r.res = 0));
  Files.Write(r, 255); Files.WriteInt(r, -9223372036854775808); Files.WriteReal(r, 3.5);
  Files.WriteNum(r, -9223372036854775808); Files.WriteNum(r, 9223372036854775807);
  Files.WriteString(r, "hello"); Files.WriteSet(r, {0, 63}); Files.WriteBool(r, TRUE);
  bytes[0] := 1; bytes[1] := 2; bytes[2] := 3; bytes[3] := 4;
  Files.WriteBytes(r, bytes, 3); ASSERT(r.res = 0);
  Files.Close(f); ASSERT(Files.Length(f) > 0);
  Files.Register(f); old := Files.Old("values.bin"); ASSERT(old # NIL);
  Files.GetDate(old, t, d);
  ASSERT((t DIV 4096 >= 0) & (t DIV 4096 < 24));
  ASSERT((t DIV 64 MOD 64 >= 0) & (t DIV 64 MOD 64 < 60));
  ASSERT((t MOD 64 >= 0) & (t MOD 64 < 60));
  ASSERT((d DIV 512 > 0) & (d DIV 32 MOD 16 >= 1) & (d DIV 32 MOD 16 <= 12) & (d MOD 32 >= 1) & (d MOD 32 <= 31));
  Files.Set(r, old, 0);
  Files.Read(r, b); ASSERT((b = 255) & ~r.eof & (r.res = 0));
  Files.ReadInt(r, i); ASSERT(i = -9223372036854775808);
  Files.ReadReal(r, x); ASSERT(x = 3.5);
  Files.ReadNum(r, i); ASSERT(i = -9223372036854775808);
  Files.ReadNum(r, i); ASSERT(i = 9223372036854775807);
  Files.ReadString(r, text); ASSERT(text = "hello");
  Files.ReadSet(r, set); ASSERT(set = {0, 63});
  Files.ReadBool(r, flag); ASSERT(flag);
  bytes[0] := 0; bytes[1] := 0; bytes[2] := 0; bytes[3] := 0;
  Files.ReadBytes(r, bytes, 4); ASSERT(r.eof & (r.res = 1) & (bytes[0] = 1) & (bytes[2] = 3));
  Files.ReadInt(r, i); ASSERT(r.eof & (r.res = 8));
  Files.GetError(error); ASSERT(error # "");
  Files.Delete("values.bin", res); ASSERT(res = 0);
  Files.Set(r, old, 0); Files.Read(r, b); ASSERT(b = 255);
  Files.Purge(old); ASSERT(Files.Length(old) = 0);
  f := Files.New("rename.bin"); Files.Register(f);
  Files.Rename("rename.bin", "renamed.bin", res); ASSERT(res = 0);
  Files.Delete("renamed.bin", res); ASSERT(res = 0);
  f := Files.New("overwrite.bin"); Files.Set(r, f, 0); Files.Write(r, 1); Files.Register(f);
  f := Files.New("overwrite.bin"); Files.Set(r, f, 0); Files.Write(r, 1); Files.Write(r, 2); Files.Register(f);
  old := Files.Old("overwrite.bin"); ASSERT(Files.Length(old) = 2); Files.Delete("overwrite.bin", res); ASSERT(res = 0);
  f := Files.New("extended.bin"); Files.Set(extended, f, 0); Files.Write(extended, 1); ASSERT(Files.Pos(extended) = 1);
  Out.String("ok"); Out.Ln
END LibraryFiles.
