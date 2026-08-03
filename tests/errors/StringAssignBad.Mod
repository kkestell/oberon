MODULE StringAssignBad;
  VAR a4: ARRAY 4 OF CHAR;
      nums: ARRAY 4 OF INTEGER;
      grid: ARRAY 2, 4 OF CHAR;
      ch: CHAR;
BEGIN
  a4 := "abcd";
  a4 := "abcdef";
  nums := "abc";
  grid := "abc";
  ch := "ab"
END StringAssignBad.
