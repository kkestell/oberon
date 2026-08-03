MODULE ArrayIndexHigh;
  IMPORT Out;
  VAR a: ARRAY 4 OF INTEGER;
      i: INTEGER;
BEGIN
  i := LEN(a);
  Out.Int(a[i], 0)
END ArrayIndexHigh.
