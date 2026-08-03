MODULE FloorNan;
  (* A NaN fails the domain test on finiteness: no comparison it takes part
     in would reject it. *)
  IMPORT Out;
  VAR x, zero: REAL;
BEGIN
  zero := 0.0; x := zero / zero;
  Out.Int(FLOOR(x), 4)
END FloorNan.
