MODULE FilesWriteBytesCount;
IMPORT Files;
VAR f: Files.File; r: Files.Rider; b: ARRAY 1 OF BYTE;
BEGIN
  f := Files.New("x"); Files.Set(r, f, 0);
  Files.WriteBytes(r, b, 2)
END FilesWriteBytesCount.
