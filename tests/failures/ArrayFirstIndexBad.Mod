MODULE ArrayFirstIndexBad;
  (* The first index is out of bounds and the second expression would trap on
     its own. The bounds line rather than the division line is what proves the
     first check ran before the second expression was evaluated. *)
  IMPORT Out;
  VAR g: ARRAY 2, 2 OF INTEGER;
      bad, zero: INTEGER;
BEGIN
  bad := 2; zero := 0;
  Out.Int(g[bad, 1 DIV zero], 0)
END ArrayFirstIndexBad.
