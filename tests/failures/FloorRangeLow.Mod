MODULE FloorRangeLow;
  (* A dynamic REAL below MIN(INTEGER) has no FLOOR result. *)
  IMPORT Out;
  VAR x: REAL;
BEGIN
  x := -3.0E9;
  Out.Int(FLOOR(x), 4)
END FloorRangeLow.
