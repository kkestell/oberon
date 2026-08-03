MODULE RealBuiltinBad;
  (* Report 10.2: ABS takes an INTEGER or a REAL, FLT takes an INTEGER, and
     FLOOR takes a REAL. PACK and UNPK are proper procedures that change
     their arguments, so those arguments must be writable variables of
     exactly the right type, and neither procedure yields a value. *)
  CONST C = 1.5;
  VAR x: REAL; n: INTEGER; b: BOOLEAN;
BEGIN
  n := ABS(b);
  x := FLT(x);
  n := FLOOR(n);
  n := FLOOR(x, x);
  x := FLT();
  PACK(C, 1);
  PACK(x + x, 1);
  PACK(n, 1);
  PACK(x, x);
  PACK(x);
  UNPK(x, x);
  UNPK(n, n);
  UNPK(C, n);
  UNPK(x, n, n);
  x := PACK(x, 1);
  n := UNPK(x, n)
END RealBuiltinBad.
