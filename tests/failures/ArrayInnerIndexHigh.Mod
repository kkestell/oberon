MODULE ArrayInnerIndexHigh;
  (* The dimensions differ, so an index that is fine for the outer one is out
     of bounds for the inner one. Only the inner dimension's own length can
     catch this, which is why every checked index carries its own. *)
  IMPORT Out;
  VAR g: ARRAY 5, 2 OF INTEGER;
      i: INTEGER;
BEGIN
  i := 4;
  Out.Int(g[i, i], 0)
END ArrayInnerIndexHigh.
