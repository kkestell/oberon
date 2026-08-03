MODULE RealIeee;
  (* The IEEE 754 binary32 behaviour this target commits to, folded and
     computed. The two columns of each row must be identical.

     Oberon source has no spelling for an infinity or a NaN, so both are
     produced by arithmetic. Overflow and division by zero are ordinary
     results here and not traps, because the Report defines no trap for them
     and QBE already gives consistent binary32 behaviour. A NaN is unequal to
     every value including itself, and every ordering comparison involving
     one is false. *)
  IMPORT Out;
  CONST
    Infinite = 1.0 / 0.0;
    Overflows = 1.0E30 * 1.0E30 = Infinite;
    Divides = 2.0 / 0.0 = Infinite;
    Signed = -1.0 / 0.0 < 0.0;
    NanEqual = 0.0 / 0.0 = 0.0 / 0.0;
    NanDiffers = 0.0 / 0.0 # 0.0 / 0.0;
    NanLess = 0.0 / 0.0 < 1.0;
    NanAtMost = 0.0 / 0.0 <= 1.0;
    NanGreater = 0.0 / 0.0 > 1.0;
    NanAtLeast = 0.0 / 0.0 >= 1.0;
  VAR one, zero, big: REAL;

BEGIN
  one := 1.0; zero := 0.0; big := 1.0E30;
  Out.Int(ORD(Overflows), 10);  Out.Int(ORD(big * big = Infinite), 10); Out.Ln;
  Out.Int(ORD(Divides), 10);    Out.Int(ORD(one / zero = Infinite), 10); Out.Ln;
  Out.Int(ORD(Signed), 10);     Out.Int(ORD(-one / zero < 0.0), 10); Out.Ln;
  Out.Int(ORD(NanEqual), 10);
  Out.Int(ORD(zero / zero = zero / zero), 10); Out.Ln;
  Out.Int(ORD(NanDiffers), 10);
  Out.Int(ORD(zero / zero # zero / zero), 10); Out.Ln;
  Out.Int(ORD(NanLess), 10);    Out.Int(ORD(zero / zero < one), 10); Out.Ln;
  Out.Int(ORD(NanAtMost), 10);  Out.Int(ORD(zero / zero <= one), 10); Out.Ln;
  Out.Int(ORD(NanGreater), 10); Out.Int(ORD(zero / zero > one), 10); Out.Ln;
  Out.Int(ORD(NanAtLeast), 10); Out.Int(ORD(zero / zero >= one), 10); Out.Ln
END RealIeee.
