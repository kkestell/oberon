MODULE ShiftRangeLow;
  (* The count comes from a variable, so the check happens at run time.
     Separate from ShiftRangeHigh because the trap ends the process. *)
  IMPORT Out;
  VAR n, count: INTEGER;
BEGIN
  n := 1; count := -1;
  Out.Int(LSL(n, count), 4)
END ShiftRangeLow.
