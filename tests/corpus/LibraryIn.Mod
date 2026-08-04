MODULE LibraryIn;

IMPORT In, Out;

VAR ch: CHAR; i: INTEGER; x: REAL; s: ARRAY 8 OF CHAR; small: ARRAY 4 OF CHAR; one: ARRAY 1 OF CHAR; zero: ARRAY 0 OF CHAR;

BEGIN
  In.Open; ASSERT(In.Done);
  In.Char(ch); ASSERT(In.Done & (ch = "A"));
  In.Open; ASSERT(~In.Done);
  In.Int(i); ASSERT(In.Done & (i = -2147483647 - 1));
  In.Int(i); ASSERT(In.Done & (i = -1));
  i := 99; In.Int(i); ASSERT(~In.Done & (i = 99));
  In.Int(i); ASSERT(In.Done & (i = 42));
  In.Real(x); ASSERT(In.Done & (x = 125.0));
  x := 7.0; In.Real(x); ASSERT(~In.Done & (x = 7.0));
  In.Real(x); ASSERT(In.Done & (x = 2.5));
  In.String(s); ASSERT(In.Done & (s = ""));
  In.String(s); ASSERT(In.Done & (s = "abc"));
  In.String(s); ASSERT(In.Done & (s = "A"));
  In.String(s); ASSERT(~In.Done & (s = ""));
  In.String(one); ASSERT(In.Done & (one = ""));
  In.String(zero); ASSERT(~In.Done);
  In.Name(small); ASSERT(~In.Done & (small = ""));
  In.Line(s); ASSERT(In.Done & (s = ""));
  In.Line(small); ASSERT(~In.Done & (small = "lin"));
  In.Line(s); ASSERT(In.Done & (s = "last"));
  In.Line(s); ASSERT(~In.Done);
  ch := "Z"; In.Char(ch); ASSERT(~In.Done & (ch = "Z"));
  Out.String("ok"); Out.Ln
END LibraryIn.
