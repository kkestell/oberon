MODULE SetRangeHigh;
  IMPORT Out;
  VAR low, high: INTEGER; s: SET;
BEGIN
  low := 0; high := 64; s := {low .. high};
  Out.Int(ORD(s), 4)
END SetRangeHigh.
