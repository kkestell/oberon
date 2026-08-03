MODULE FloorRangeHigh;
  (* MAX(INTEGER) rounds up to 2147483648.0 in binary32, which is one value
     past the top of the FLOOR domain. *)
  IMPORT Out;
  VAR x: REAL; n: INTEGER;
BEGIN
  n := 2147483647; x := FLT(n);
  Out.Int(FLOOR(x), 4)
END FloorRangeHigh.
