MODULE WideSets;
  (* SET holds elements 0 through 63. Every value is computed twice: once
     folded into a constant declaration and once at run time from variables
     holding the same operands. The two columns of each row must be
     identical. ORD reinterprets the 64-bit vector as a signed INTEGER, so a
     set holding 63 prints negative. *)
  IMPORT Out;
  CONST
    High = {32 .. 63};
    Crossing = {30 .. 33};
    Low = -High;
    Top = {63};
    Ends = {0, 63};
    HasTop = 63 IN Ends;
    HasBottom = 0 IN Ends;
    HasMiddle = 32 IN Ends;
  VAR s: SET; low, high: INTEGER;

BEGIN
  low := 32; high := 63;
  Out.Int(ORD(High), 22);      Out.Int(ORD({low .. high}), 22); Out.Ln;
  low := 30; high := 33;
  Out.Int(ORD(Crossing), 22);  Out.Int(ORD({low .. high}), 22); Out.Ln;
  s := {32 .. 63};
  Out.Int(ORD(Low), 22);       Out.Int(ORD(-s), 22); Out.Ln;
  high := 63;
  Out.Int(ORD(Top), 22);       Out.Int(ORD({high}), 22); Out.Ln;
  s := {0}; INCL(s, high);
  Out.Int(ORD(Ends), 22);      Out.Int(ORD(s), 22); Out.Ln;
  Out.Int(ORD(HasTop), 22);    Out.Int(ORD(high IN s), 22); Out.Ln;
  low := 0;
  Out.Int(ORD(HasBottom), 22); Out.Int(ORD(low IN s), 22); Out.Ln;
  low := 32;
  Out.Int(ORD(HasMiddle), 22); Out.Int(ORD(low IN s), 22); Out.Ln;
  EXCL(s, high);
  Out.Int(ORD(Ends - Top), 22); Out.Int(ORD(s), 22); Out.Ln
END WideSets.
