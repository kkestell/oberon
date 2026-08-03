MODULE RealBuiltins;
  (* Report 10.2 ABS, FLT, and FLOOR, folded and computed. The two columns of
     each row must be identical.

     FLOOR rounds toward negative infinity, so FLOOR(-1.5) is -2 and not -1.
     Signed zero has no printable form, so the two zeros are told apart by
     dividing into them: 1.0 / +0.0 is a positive infinity and 1.0 / -0.0 a
     negative one. FLT rounds to the nearest binary32 value, so 2^24 survives
     the conversion and 2^24 + 1 does not. *)
  IMPORT Out;
  CONST
    NegZero = -0.;
    Abs = ABS(-1.5);
    AbsClearsSign = 1.0 / ABS(NegZero) > 0.0;
    NegZeroKeepsSign = 1.0 / NegZero < 0.0;
    FloorDown = FLOOR(-1.5);
    FloorUp = FLOOR(1.5);
    (* MIN(INTEGER) is a power of two, so it is the one endpoint of the FLOOR
       domain that a REAL can hold exactly. *)
    FloorMin = FLOOR(FLT(-2147483648));
    Flt = FLT(-3);
    FltExact = FLOOR(FLT(16777216)) = 16777216;
    FltRounds = FLT(16777217) = FLT(16777216);
  VAR r, zero: REAL; n: INTEGER;

BEGIN
  r := -1.5; zero := -0.;
  Out.Int(FLOOR(Abs), 14);          Out.Int(FLOOR(ABS(r)), 14); Out.Ln;
  Out.Int(ORD(AbsClearsSign), 14);  Out.Int(ORD(1.0 / ABS(zero) > 0.0), 14);
  Out.Ln;
  Out.Int(ORD(NegZeroKeepsSign), 14); Out.Int(ORD(1.0 / zero < 0.0), 14);
  Out.Ln;
  Out.Int(FloorDown, 14);           Out.Int(FLOOR(r), 14); Out.Ln;
  Out.Int(FloorUp, 14);             r := 1.5; Out.Int(FLOOR(r), 14); Out.Ln;
  Out.Int(FloorMin, 14);            n := -2147483648;
                                    Out.Int(FLOOR(FLT(n)), 14); Out.Ln;
  Out.Int(FLOOR(Flt), 14);          n := -3;
                                    Out.Int(FLOOR(FLT(n)), 14); Out.Ln;
  Out.Int(ORD(FltExact), 14);       n := 16777216;
                                    Out.Int(ORD(FLOOR(FLT(n)) = n), 14);
  Out.Ln;
  Out.Int(ORD(FltRounds), 14);      n := 16777217;
                                    Out.Int(ORD(FLT(n) = FLT(16777216)), 14);
  Out.Ln
END RealBuiltins.
