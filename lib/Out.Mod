MODULE Out;

IMPORT OberonRuntime;

PROCEDURE Open*;
END Open;

PROCEDURE Char*(ch: CHAR);
BEGIN
  OberonRuntime.OutChar(ch)
END Char;

PROCEDURE String*(s: ARRAY OF CHAR);
BEGIN
  OberonRuntime.OutString(s)
END String;

PROCEDURE Int*(i, n: INTEGER);
BEGIN
  OberonRuntime.OutInt(i, n)
END Int;

PROCEDURE Hex*(i: INTEGER);
BEGIN
  OberonRuntime.OutHex(i)
END Hex;

PROCEDURE Real*(x: REAL; n: INTEGER);
BEGIN
  OberonRuntime.OutReal(x, n)
END Real;

PROCEDURE Ln*;
BEGIN
  OberonRuntime.OutLn
END Ln;

END Out.
