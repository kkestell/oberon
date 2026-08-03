MODULE StringUseBad;
  VAR i: INTEGER; b: BOOLEAN; ch: CHAR;

  PROCEDURE Take(VAR c: CHAR);
  BEGIN
    c := "x"
  END Take;

BEGIN
  i := "ab" + 1;
  b := "ab" = 1;
  b := "a" = 1;
  Take("x");
  i := LEN("abc");
  ch := 0X; b := "" = ch;
  i := ORD("ab")
END StringUseBad.
