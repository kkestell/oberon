MODULE ArrayZeroIndex;
  (* A zero-length array has no element at any index, so the check fails even
     for zero. Nothing is loaded and no pointer arithmetic decides this. *)
  IMPORT Out;
  VAR empty: ARRAY 0 OF INTEGER;
      i: INTEGER;
BEGIN
  Out.Int(LEN(empty), 0); Out.Ln;
  i := 0;
  empty[i] := 1
END ArrayZeroIndex.
