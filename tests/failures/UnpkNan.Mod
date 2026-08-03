MODULE UnpkNan;
  (* Neither has a NaN. *)
  IMPORT Out;
  VAR x, zero: REAL; n: INTEGER;
BEGIN
  zero := 0.0; x := zero / zero;
  UNPK(x, n);
  Out.Int(n, 4)
END UnpkNan.
