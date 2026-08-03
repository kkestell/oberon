MODULE Bytes;
  (* Report 6.1: BYTE holds the integers 0 through 255 and is compatible with
     INTEGER in both directions. Reading a BYTE yields an INTEGER, so the
     arithmetic below is ordinary INTEGER arithmetic; writing one checks the
     range, and every value here stays inside it. *)
  IMPORT Out;
  VAR b, c: BYTE;
      arr: ARRAY 4 OF BYTE;
      i: INTEGER;

  PROCEDURE Sum(x: BYTE; n: INTEGER): INTEGER;
  BEGIN
  RETURN x + n
  END Sum;

  PROCEDURE Halve(VAR v: BYTE);
  BEGIN
    v := v DIV 2
  END Halve;

  PROCEDURE Limit(): BYTE;
    VAR t: BYTE;
  BEGIN
    t := 255
  RETURN t
  END Limit;

BEGIN
  (* both boundary values *)
  b := 0; c := 255;
  Out.Int(b, 6); Out.Int(c, 6); Out.Ln;
  (* assignment in both directions between INTEGER and BYTE *)
  i := c;
  b := i - 155;
  Out.Int(b, 6); Out.Int(i, 6); Out.Ln;
  (* arithmetic mixing the two *)
  i := b * 2 + c - 255;
  Out.Int(i, 6); Out.Ln;
  (* INC and DEC write through the same checked store *)
  INC(b); Out.Int(b, 6);
  DEC(b, 2); Out.Int(b, 6); Out.Ln;
  (* a BYTE value parameter, from a BYTE variable and an INTEGER constant *)
  Out.Int(Sum(b, 1), 6); Out.Int(Sum(200, 55), 6); Out.Ln;
  (* a BYTE variable parameter and a BYTE function result *)
  Halve(c); Out.Int(c, 6); Out.Ln;
  b := Limit(); Out.Int(b, 6); Out.Ln;
  (* an array of BYTE is one byte per element *)
  FOR i := 0 TO LEN(arr) - 1 DO arr[i] := i * 80 END;
  i := arr[1] + arr[3];
  Out.Int(i, 6); Out.Int(arr[2], 6); Out.Ln
END Bytes.
