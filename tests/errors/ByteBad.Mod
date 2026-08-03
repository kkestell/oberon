MODULE ByteBad;
  VAR b: BYTE; i: INTEGER; r: REAL;

  PROCEDURE VarInt(VAR n: INTEGER);
  BEGIN
    n := 0
  END VarInt;

  PROCEDURE VarByte(VAR n: BYTE);
  BEGIN
    n := 0
  END VarByte;

BEGIN
  b := 256;
  b := -1;
  VarByte(i);
  VarInt(b);
  FOR b := 0 TO 3 DO END;
  r := b
END ByteBad.
