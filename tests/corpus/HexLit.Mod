MODULE HexLit;
  (* A hex literal is a 64-bit pattern, not a magnitude, so 0FFFFFFFFFFFFFFFFH
     is -1 rather than an out-of-range error, and eight digits with the top
     bit set are an ordinary positive value. The last case covers the
     negative literal that has no positive counterpart. *)
  IMPORT Out;
  CONST
    Word = 0FFFFFFFFH;
    Neg1 = 0FFFFFFFFFFFFFFFFH;
    Least = 08000000000000000H;
    Greatest = 07FFFFFFFFFFFFFFFH;
  VAR x: INTEGER;
BEGIN
  Out.Int(Word, 0); Out.Ln;
  Out.Int(Neg1, 0); Out.Ln;
  Out.Int(Least, 0); Out.Ln;
  Out.Int(Greatest, 0); Out.Ln;
  x := -9223372036854775808;
  Out.Int(x, 0); Out.Ln
END HexLit.
