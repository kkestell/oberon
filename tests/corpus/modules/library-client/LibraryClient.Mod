MODULE LibraryClient;

IMPORT LibrarySupport, Out;

TYPE Printer = PROCEDURE(s: ARRAY OF CHAR);
VAR print: Printer; f: LibrarySupport.File; r: LibrarySupport.Rider;

BEGIN
  ASSERT(LibrarySupport.tau > 6.0);
  ASSERT(LibrarySupport.TextLength("ok") = 2);
  ASSERT(LibrarySupport.Unit(1.0) = 1.0);
  print := LibrarySupport.Print; print("ok"); Out.Ln;
  f := NIL; r.eof := FALSE; r.res := 0
END LibraryClient.
