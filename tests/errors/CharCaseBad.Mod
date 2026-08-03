MODULE CharCaseBad;
  CONST C = CHR(97);
  VAR ch: CHAR; i: INTEGER;
BEGIN
  CASE ch OF
    1: i := 0
  | "ab": i := 1
  | "z" .. "a": i := 2
  | "b" .. "j": i := 3
  | "e" .. "g": i := 4
  END;
  CASE i OF
    C: i := 5
  | "a": i := 6
  END
END CharCaseBad.
