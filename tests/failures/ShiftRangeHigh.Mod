MODULE ShiftRangeHigh;
  (* QBE reduces a shift count modulo the result width, which would silently
     turn a count of 32 into a count of 0. Report the error instead. *)
  IMPORT Out;
  VAR n, count: INTEGER;
BEGIN
  n := 1; count := 32;
  Out.Int(ASR(n, count), 4)
END ShiftRangeHigh.
