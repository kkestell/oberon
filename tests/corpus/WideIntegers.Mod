MODULE WideIntegers;
  (* INTEGER is signed 64-bit. Every value is computed twice: once folded into
     a constant declaration and once at run time from variables holding the
     same operands. The two columns of each row must be identical. *)
  IMPORT Out;
  CONST
    Min = -9223372036854775808;
    Product = 3000000000 * 3;
    Sum = 4294967295 + 1;
    (* Named, because a leading minus applies to the whole term:
       -10000000001 DIV 3 is -(10000000001 DIV 3). *)
    Negative = -10000000001;
    Quotient = Negative DIV 3;
    Remainder = Negative MOD 3;
    Top = LSL(1, 63);
    Sign = ASR(Min, 63);
    Rotated = ROR(1, 40);
    Absolute = ABS(-5000000000);
    Floored = FLOOR(1.0E15);
    (* 2^53 is the largest power of two below which every INTEGER converts
       to REAL exactly, so FLT and FLOOR return it unchanged. *)
    Exact = FLOOR(FLT(9007199254740992));
  VAR a, b, n: INTEGER; x: REAL;

BEGIN
  a := 3000000000; b := 3;
  Out.Int(Product, 21);   Out.Int(a * b, 21); Out.Ln;
  a := 4294967295; b := 1;
  Out.Int(Sum, 21);       Out.Int(a + b, 21); Out.Ln;
  a := -10000000001; b := 3;
  Out.Int(Quotient, 21);  Out.Int(a DIV b, 21); Out.Ln;
  Out.Int(Remainder, 21); Out.Int(a MOD b, 21); Out.Ln;
  n := 1;
  Out.Int(Top, 21);       Out.Int(LSL(n, 63), 21); Out.Ln;
  n := Min;
  Out.Int(Sign, 21);      Out.Int(ASR(n, 63), 21); Out.Ln;
  n := 1;
  Out.Int(Rotated, 21);   Out.Int(ROR(n, 40), 21); Out.Ln;
  n := -5000000000;
  Out.Int(Absolute, 21);  Out.Int(ABS(n), 21); Out.Ln;
  x := 1.0E15;
  Out.Int(Floored, 21);   Out.Int(FLOOR(x), 21); Out.Ln;
  n := 9007199254740992;
  Out.Int(Exact, 21);     Out.Int(FLOOR(FLT(n)), 21); Out.Ln
END WideIntegers.
