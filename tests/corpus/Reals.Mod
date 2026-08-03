MODULE Reals;
  (* Report 3, 8.2.2, and 8.2.4. Every value is computed twice: once folded
     into a constant declaration and once at run time from variables holding
     the same operands. The two columns of each row must be identical, as in
     Sets.Mod. A REAL reaches the output through FLOOR, which is the only
     conversion to INTEGER there is until the portable library arrives.

     Every literal here is exact in binary32, so no row depends on rounding.
     Small is 12.5 rather than an integer so the rows also show FLOOR
     rounding toward negative infinity on both signs. *)
  IMPORT Out;
  CONST
    Two = 2.;         (* no fraction digits *)
    Big = 2.5E3;      (* positive scale factor *)
    Small = 125.E-1;  (* negative scale factor *)
    Plus = +Small;
    Neg = -Small;
    Sum = Big + Small;
    Diff = Big - Small;
    Prod = Small * Two;
    Quot = Big / Small;
    Equal = Small = 12.5;
    Differs = Small # Two;
    Less = Small < Big;
    AtMost = Two <= Two;
    Greater = Small > Big;
    AtLeast = Two >= Big;
  VAR two, big, small: REAL;

BEGIN
  two := 2.; big := 2.5E3; small := 125.E-1;
  Out.Int(FLOOR(Two), 10);   Out.Int(FLOOR(two), 10); Out.Ln;
  Out.Int(FLOOR(Big), 10);   Out.Int(FLOOR(big), 10); Out.Ln;
  Out.Int(FLOOR(Small), 10); Out.Int(FLOOR(small), 10); Out.Ln;
  Out.Int(FLOOR(Plus), 10);  Out.Int(FLOOR(+small), 10); Out.Ln;
  Out.Int(FLOOR(Neg), 10);   Out.Int(FLOOR(-small), 10); Out.Ln;
  Out.Int(FLOOR(Sum), 10);   Out.Int(FLOOR(big + small), 10); Out.Ln;
  Out.Int(FLOOR(Diff), 10);  Out.Int(FLOOR(big - small), 10); Out.Ln;
  Out.Int(FLOOR(Prod), 10);  Out.Int(FLOOR(small * two), 10); Out.Ln;
  Out.Int(FLOOR(Quot), 10);  Out.Int(FLOOR(big / small), 10); Out.Ln;
  Out.Int(ORD(Equal), 10);   Out.Int(ORD(small = 12.5), 10); Out.Ln;
  Out.Int(ORD(Differs), 10); Out.Int(ORD(small # two), 10); Out.Ln;
  Out.Int(ORD(Less), 10);    Out.Int(ORD(small < big), 10); Out.Ln;
  Out.Int(ORD(AtMost), 10);  Out.Int(ORD(two <= two), 10); Out.Ln;
  Out.Int(ORD(Greater), 10); Out.Int(ORD(small > big), 10); Out.Ln;
  Out.Int(ORD(AtLeast), 10); Out.Int(ORD(two >= big), 10); Out.Ln
END Reals.
