MODULE LibraryProfileBad;
IMPORT Files, In, Out;
TYPE Hidden = Files.Handle;
  Wrong = PROCEDURE(VAR value: ARRAY OF BYTE);
VAR r: Files.Rider; p: Wrong; bytes: ARRAY 2 OF BYTE;
BEGIN
  r.pos := 0;
  r.base := NIL;
  In.Done := FALSE;
  Out.String(bytes);
  p := In.String
END LibraryProfileBad.
