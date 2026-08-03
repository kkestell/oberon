MODULE ArrayIndexLow;
  IMPORT Out;
  VAR a: ARRAY 4 OF INTEGER;
      i: INTEGER;
BEGIN
  i := -1;
  a[i] := 1;
  Out.Int(a[i], 0)
END ArrayIndexLow.
