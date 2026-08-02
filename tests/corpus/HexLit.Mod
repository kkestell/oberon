MODULE HexLit;
  (* A hex literal is a 32-bit pattern, not a magnitude, so 0FFFFFFFFH is -1
     rather than an out-of-range error. The last case covers the negative
     literal that has no positive counterpart. *)
  IMPORT Out;
  CONST Neg1 = 0FFFFFFFFH; Least = 080000000H; Greatest = 07FFFFFFFH;
  VAR x: INTEGER;
BEGIN
  Out.Int(Neg1, 0); Out.Ln;
  Out.Int(Least, 0); Out.Ln;
  Out.Int(Greatest, 0); Out.Ln;
  x := -2147483648;
  Out.Int(x, 0); Out.Ln
END HexLit.
