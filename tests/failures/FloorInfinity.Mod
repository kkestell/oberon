MODULE FloorInfinity;
  (* Dividing by zero is an ordinary REAL result, so the trap belongs to
     FLOOR and not to the division. *)
  IMPORT Out;
  VAR x, zero: REAL;
BEGIN
  x := 1.0; zero := 0.0; x := x / zero;
  Out.Int(FLOOR(x), 4)
END FloorInfinity.
