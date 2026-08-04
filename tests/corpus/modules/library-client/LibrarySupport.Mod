MODULE LibrarySupport;

IMPORT Out, Math, Strings, Files;

CONST tau* = 2.0 * Math.pi;
TYPE File* = Files.File; Rider* = Files.Rider;

PROCEDURE Print*(s: ARRAY OF CHAR);
BEGIN
  Out.String(s)
END Print;

PROCEDURE TextLength*(s: ARRAY OF CHAR): INTEGER;
RETURN Strings.Length(s)
END TextLength;

PROCEDURE Unit*(x: REAL): REAL;
RETURN Math.sqrt(x)
END Unit;

BEGIN
  Out.Char("i")
END LibrarySupport.
