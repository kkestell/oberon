MODULE FilesInvalidPosition;
IMPORT Files;
VAR f: Files.File; r: Files.Rider;
BEGIN
  f := Files.New("x");
  Files.Set(r, f, 1)
END FilesInvalidPosition.
