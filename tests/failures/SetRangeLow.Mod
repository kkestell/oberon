MODULE SetRangeLow;
  (* The range is reversed and would be empty, but its low endpoint is still
     checked: validation comes before the empty-range result is selected. *)
  IMPORT Out;
  VAR low, high: INTEGER; s: SET;
BEGIN
  low := 64; high := 0; s := {low .. high};
  Out.Int(ORD(s), 4)
END SetRangeLow.
