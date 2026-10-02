MODULE FloorRangeHigh;
  (* MAX(INTEGER) rounds up to 9223372036854775808.0 in binary64, which is one value
     past the top of the FLOOR domain. *)
  IMPORT Out;
  VAR x: REAL; n: INTEGER;
BEGIN
  n := 9223372036854775807; x := FLT(n);
  Out.Int(FLOOR(x), 4)
END FloorRangeHigh.
