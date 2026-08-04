MODULE Files;

IMPORT OberonRuntime;

TYPE
  File* = POINTER TO Handle;
  Handle = RECORD id: INTEGER END;
  Rider* = RECORD
    eof*: BOOLEAN;
    res*: INTEGER;
    base: File;
    pos: INTEGER
  END;

PROCEDURE Terminated(s: ARRAY OF CHAR): BOOLEAN;
  VAR i: INTEGER;
BEGIN
  i := 0;
  WHILE (i < LEN(s)) & (s[i] # 0X) DO INC(i) END
RETURN i < LEN(s)
END Terminated;

PROCEDURE Bound(VAR r: Rider);
BEGIN
  ASSERT(r.base # NIL)
END Bound;

PROCEDURE Old*(name: ARRAY OF CHAR): File;
  VAR id: INTEGER; f: File;
BEGIN
  ASSERT(Terminated(name));
  id := OberonRuntime.FileOld(name);
  IF id # 0 THEN
    NEW(f);
    IF f = NIL THEN OberonRuntime.FileRelease(id) ELSE f.id := id END
  ELSE
    f := NIL
  END
RETURN f
END Old;

PROCEDURE New*(name: ARRAY OF CHAR): File;
  VAR id: INTEGER; f: File;
BEGIN
  ASSERT(Terminated(name));
  id := OberonRuntime.FileNew(name);
  IF id # 0 THEN
    NEW(f);
    IF f = NIL THEN OberonRuntime.FileRelease(id) ELSE f.id := id END
  ELSE
    f := NIL
  END
RETURN f
END New;

PROCEDURE Register*(f: File);
BEGIN
  ASSERT(f # NIL);
  OberonRuntime.FileRegister(f.id)
END Register;

PROCEDURE Close*(f: File);
BEGIN
  ASSERT(f # NIL);
  OberonRuntime.FileClose(f.id)
END Close;

PROCEDURE Purge*(f: File);
BEGIN
  ASSERT(f # NIL);
  OberonRuntime.FilePurge(f.id)
END Purge;

PROCEDURE Delete*(name: ARRAY OF CHAR; VAR res: INTEGER);
BEGIN
  ASSERT(Terminated(name));
  res := OberonRuntime.FileDelete(name)
END Delete;

PROCEDURE Rename*(old, new: ARRAY OF CHAR; VAR res: INTEGER);
BEGIN
  ASSERT(Terminated(old));
  ASSERT(Terminated(new));
  res := OberonRuntime.FileRename(old, new)
END Rename;

PROCEDURE Length*(f: File): INTEGER;
BEGIN
  ASSERT(f # NIL)
RETURN OberonRuntime.FileLength(f.id)
END Length;

PROCEDURE GetDate*(f: File; VAR t, d: INTEGER);
  VAR done: BOOLEAN;
BEGIN
  ASSERT(f # NIL);
  done := OberonRuntime.FileDate(f.id, t, d)
END GetDate;

PROCEDURE Set*(VAR r: Rider; f: File; pos: INTEGER);
BEGIN
  ASSERT(f # NIL);
  ASSERT(pos >= 0);
  ASSERT(pos <= Length(f));
  r.base := f;
  r.pos := pos;
  r.eof := FALSE;
  r.res := 0
END Set;

PROCEDURE Pos*(VAR r: Rider): INTEGER;
BEGIN
  Bound(r)
RETURN r.pos
END Pos;

PROCEDURE Base*(VAR r: Rider): File;
BEGIN
  Bound(r)
RETURN r.base
END Base;

PROCEDURE Read*(VAR r: Rider; VAR x: BYTE);
  VAR status, moved: INTEGER;
BEGIN
  Bound(r);
  status := OberonRuntime.FileReadByte(r.base.id, r.pos, x, moved);
  r.pos := r.pos + moved;
  r.eof := status = 1;
  r.res := 1 - moved
END Read;

PROCEDURE ReadInt*(VAR r: Rider; VAR i: INTEGER);
  VAR status, moved: INTEGER;
BEGIN
  Bound(r);
  status := OberonRuntime.FileReadInt(r.base.id, r.pos, i, moved);
  r.pos := r.pos + moved;
  r.eof := status = 1;
  r.res := 4 - moved
END ReadInt;

PROCEDURE ReadReal*(VAR r: Rider; VAR x: REAL);
  VAR status, moved: INTEGER;
BEGIN
  Bound(r);
  status := OberonRuntime.FileReadReal(r.base.id, r.pos, x, moved);
  r.pos := r.pos + moved;
  r.eof := status = 1;
  r.res := 4 - moved
END ReadReal;

PROCEDURE ReadNum*(VAR r: Rider; VAR i: INTEGER);
  VAR status, moved: INTEGER;
BEGIN
  Bound(r);
  status := OberonRuntime.FileReadNum(r.base.id, r.pos, i, moved);
  r.pos := r.pos + moved;
  r.eof := status = 1;
  IF status = 0 THEN r.res := 0 ELSE r.res := 1 END
END ReadNum;

PROCEDURE ReadString*(VAR r: Rider; VAR s: ARRAY OF CHAR);
  VAR status, moved: INTEGER;
BEGIN
  Bound(r);
  ASSERT(LEN(s) > 0);
  status := OberonRuntime.FileReadString(r.base.id, r.pos, s, moved);
  r.pos := r.pos + moved;
  r.eof := status = 1;
  IF status = 0 THEN r.res := 0 ELSE r.res := 1 END
END ReadString;

PROCEDURE ReadSet*(VAR r: Rider; VAR s: SET);
  VAR status, moved: INTEGER;
BEGIN
  Bound(r);
  status := OberonRuntime.FileReadSet(r.base.id, r.pos, s, moved);
  r.pos := r.pos + moved;
  r.eof := status = 1;
  r.res := 4 - moved
END ReadSet;

PROCEDURE ReadBool*(VAR r: Rider; VAR b: BOOLEAN);
  VAR status, moved: INTEGER;
BEGIN
  Bound(r);
  status := OberonRuntime.FileReadBool(r.base.id, r.pos, b, moved);
  r.pos := r.pos + moved;
  r.eof := status = 1;
  IF status = 0 THEN r.res := 0 ELSE r.res := 1 END
END ReadBool;

PROCEDURE ReadBytes*(VAR r: Rider; VAR buf: ARRAY OF BYTE; n: INTEGER);
  VAR status, moved: INTEGER;
BEGIN
  Bound(r);
  ASSERT(n >= 0);
  status := OberonRuntime.FileReadBytes(r.base.id, r.pos, buf, n, moved);
  r.pos := r.pos + moved;
  r.eof := status = 1;
  r.res := n - moved
END ReadBytes;

PROCEDURE Write*(VAR r: Rider; x: BYTE);
  VAR status, moved: INTEGER;
BEGIN
  Bound(r);
  status := OberonRuntime.FileWriteByte(r.base.id, r.pos, x, moved);
  r.pos := r.pos + moved;
  r.res := 1 - moved
END Write;

PROCEDURE WriteInt*(VAR r: Rider; i: INTEGER);
  VAR status, moved: INTEGER;
BEGIN
  Bound(r);
  status := OberonRuntime.FileWriteInt(r.base.id, r.pos, i, moved);
  r.pos := r.pos + moved;
  r.res := 4 - moved
END WriteInt;

PROCEDURE WriteReal*(VAR r: Rider; x: REAL);
  VAR status, moved: INTEGER;
BEGIN
  Bound(r);
  status := OberonRuntime.FileWriteReal(r.base.id, r.pos, x, moved);
  r.pos := r.pos + moved;
  r.res := 4 - moved
END WriteReal;

PROCEDURE WriteNum*(VAR r: Rider; i: INTEGER);
  VAR status, moved: INTEGER;
BEGIN
  Bound(r);
  status := OberonRuntime.FileWriteNum(r.base.id, r.pos, i, moved);
  r.pos := r.pos + moved;
  IF status = 0 THEN r.res := 0 ELSE r.res := 1 END
END WriteNum;

PROCEDURE WriteString*(VAR r: Rider; s: ARRAY OF CHAR);
  VAR status, moved: INTEGER;
BEGIN
  Bound(r);
  ASSERT(Terminated(s));
  status := OberonRuntime.FileWriteString(r.base.id, r.pos, s, moved);
  r.pos := r.pos + moved;
  IF status = 0 THEN r.res := 0 ELSE r.res := 1 END
END WriteString;

PROCEDURE WriteSet*(VAR r: Rider; s: SET);
  VAR status, moved: INTEGER;
BEGIN
  Bound(r);
  status := OberonRuntime.FileWriteSet(r.base.id, r.pos, s, moved);
  r.pos := r.pos + moved;
  r.res := 4 - moved
END WriteSet;

PROCEDURE WriteBool*(VAR r: Rider; b: BOOLEAN);
  VAR status, moved: INTEGER;
BEGIN
  Bound(r);
  status := OberonRuntime.FileWriteBool(r.base.id, r.pos, b, moved);
  r.pos := r.pos + moved;
  r.res := 1 - moved
END WriteBool;

PROCEDURE WriteBytes*(VAR r: Rider; VAR buf: ARRAY OF BYTE; n: INTEGER);
  VAR status, moved: INTEGER;
BEGIN
  Bound(r);
  ASSERT(n >= 0);
  ASSERT(n <= LEN(buf));
  status := OberonRuntime.FileWriteBytes(r.base.id, r.pos, buf, n, moved);
  r.pos := r.pos + moved;
  r.res := n - moved
END WriteBytes;

PROCEDURE GetError*(VAR msg: ARRAY OF CHAR);
BEGIN
  OberonRuntime.FileError(msg)
END GetError;

END Files.
