MODULE In;

IMPORT OberonRuntime;

VAR Done*: BOOLEAN;

PROCEDURE Open*;
BEGIN
  Done := OberonRuntime.InOpen()
END Open;

PROCEDURE Char*(VAR ch: CHAR);
BEGIN
  Done := OberonRuntime.InChar(ch)
END Char;

PROCEDURE Int*(VAR i: INTEGER);
BEGIN
  Done := OberonRuntime.InInt(i)
END Int;

PROCEDURE Real*(VAR x: REAL);
BEGIN
  Done := OberonRuntime.InReal(x)
END Real;

PROCEDURE String*(VAR str: ARRAY OF CHAR);
BEGIN
  Done := OberonRuntime.InString(str)
END String;

PROCEDURE Name*(VAR name: ARRAY OF CHAR);
BEGIN
  Done := OberonRuntime.InName(name)
END Name;

PROCEDURE Line*(VAR line: ARRAY OF CHAR);
BEGIN
  Done := OberonRuntime.InLine(line)
END Line;

BEGIN
  Done := FALSE
END In.
