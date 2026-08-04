MODULE OpenArrayStrings;
IMPORT Out;
CONST Greeting = "abc";
VAR short: ARRAY 4 OF CHAR;
    long: ARRAY 8 OF CHAR;

PROCEDURE Bool(b: BOOLEAN);
BEGIN IF b THEN Out.Int(1, 0) ELSE Out.Int(0, 0) END
END Bool;

PROCEDURE Compare(a, b: ARRAY OF CHAR);
BEGIN
  Out.Int(LEN(a), 0); Out.Char(":"); Out.Int(LEN(b), 0); Out.Char(":");
  Bool(a = "abc"); Out.Char(":"); Bool(a = b); Out.Char(":"); Bool(a < b); Out.Ln
END Compare;

PROCEDURE Contains(a: ARRAY OF CHAR; ch: CHAR): BOOLEAN;
VAR i: INTEGER; found: BOOLEAN;
BEGIN
  i := 0; found := FALSE;
  WHILE (i < LEN(a)) & ~found DO found := a[i] = ch; INC(i) END
RETURN found
END Contains;

PROCEDURE Replace(VAR a: ARRAY OF CHAR);
BEGIN a[0] := "Z"; a := "ok"
END Replace;

BEGIN
  short := "abc"; long := "abd";
  Compare("abc", Greeting);
  Compare(short, long);
  Bool(Contains("hello", "e")); Out.Ln;
  Replace(long); Out.Int(LEN(long), 0); Out.Char(":"); Bool(long = "ok"); Out.Ln
END OpenArrayStrings.
