MODULE LibraryInOpen;

IMPORT In, Out;

VAR n: INTEGER; line: ARRAY 2 OF CHAR;

BEGIN
  In.Open; ASSERT(In.Done);
  In.Int(n); ASSERT(In.Done & (n = 7));
  In.Line(line); ASSERT(In.Done & (line = ""));
  In.Open; ASSERT(~In.Done);
  Out.String("ok"); Out.Ln
END LibraryInOpen.
