MODULE UnpkInfinity;
  (* An infinity has no normalized fraction and exponent. *)
  IMPORT Out;
  VAR x, zero: REAL; n: INTEGER;
BEGIN
  x := 1.0; zero := 0.0; x := x / zero;
  UNPK(x, n);
  Out.Int(n, 4)
END UnpkInfinity.
