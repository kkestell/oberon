MODULE CharCaseNoMatch;
  VAR ch: CHAR;
BEGIN
  ch := "x";
  CASE ch OF
    "a": ch := "y"
  END
END CharCaseNoMatch.
