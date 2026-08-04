MODULE FilesZeroReadString;
IMPORT Files;
VAR f: Files.File; r: Files.Rider; s: ARRAY 0 OF CHAR;
BEGIN
  f := Files.New("x"); Files.Set(r, f, 0);
  Files.ReadString(r, s)
END FilesZeroReadString.
