MODULE ArrayLenBad;
  VAR a: ARRAY 3 OF INTEGER;
      n: INTEGER;
BEGIN
  n := LEN(a, 1);
  n := LEN(n);
  n := LEN(INTEGER);
  n := LEN(1 + 2)
END ArrayLenBad.
