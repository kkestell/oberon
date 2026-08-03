MODULE ArrayIndexBad;
  VAR a: ARRAY 4 OF INTEGER;
      g: ARRAY 2, 2 OF INTEGER;
      n: INTEGER;
BEGIN
  n := a[TRUE];
  n := n[0];
  n := g[0, 0, 0];
  n := a[-1];
  n := a[4];
  n := LEN(a[9])
END ArrayIndexBad.
