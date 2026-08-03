MODULE Sets;
  (* Report 6.1 and 8.2. Every value is computed twice: once folded into a
     constant declaration and once at run time from variables holding the
     same operands. The two columns of each row must be identical, as in
     DivMod.Mod. SET values are printed through ORD, which reinterprets the
     32-bit vector as a signed INTEGER, so the empty set prints 0, {31}
     prints MIN(INTEGER), and the full set prints -1. *)
  IMPORT Out;
  CONST
    Empty = {};
    Full = {0 .. 31};
    Zero = {0};
    Top = {31};
    Mid = {2 .. 5};
    Reversed = {5 .. 2};
    A = {0, 2, 4};
    B = {2, 3};
    Union = A + B;
    Difference = A - B;
    Intersection = A * B;
    Symmetric = A / B;
    Complement = -A;
    Same = A = {0, 2, 4};
    Differs = A # B;
    Has2 = 2 IN A;
    Has1 = 1 IN A;
  VAR a, b, s: SET; low, high: INTEGER;

BEGIN
  a := {0, 2, 4}; b := {2, 3}; s := {0 .. 31};
  Out.Int(ORD(Empty), 14);        Out.Int(ORD({}), 14); Out.Ln;
  Out.Int(ORD(Full), 14);         Out.Int(ORD(s), 14); Out.Ln;
  Out.Int(ORD(Zero), 14);         low := 0; Out.Int(ORD({low}), 14); Out.Ln;
  Out.Int(ORD(Top), 14);          high := 31; Out.Int(ORD({high}), 14); Out.Ln;
  Out.Int(ORD(Mid), 14);          low := 2; high := 5;
                                  Out.Int(ORD({low .. high}), 14); Out.Ln;
  Out.Int(ORD(Reversed), 14);     low := 5; high := 2;
                                  Out.Int(ORD({low .. high}), 14); Out.Ln;
  Out.Int(ORD(Union), 14);        Out.Int(ORD(a + b), 14); Out.Ln;
  Out.Int(ORD(Difference), 14);   Out.Int(ORD(a - b), 14); Out.Ln;
  Out.Int(ORD(Intersection), 14); Out.Int(ORD(a * b), 14); Out.Ln;
  Out.Int(ORD(Symmetric), 14);    Out.Int(ORD(a / b), 14); Out.Ln;
  Out.Int(ORD(Complement), 14);   Out.Int(ORD(-a), 14); Out.Ln;
  Out.Int(ORD(Same), 14);         Out.Int(ORD(a = {0, 2, 4}), 14); Out.Ln;
  Out.Int(ORD(Differs), 14);      Out.Int(ORD(a # b), 14); Out.Ln;
  Out.Int(ORD(Has2), 14);         low := 2; Out.Int(ORD(low IN a), 14); Out.Ln;
  Out.Int(ORD(Has1), 14);         low := 1; Out.Int(ORD(low IN a), 14); Out.Ln;

  (* Bit 31 is an ordinary element, so complementing the full set empties it
     and removing {31} leaves MAX(INTEGER). *)
  Out.Int(ORD(-Full), 14);        Out.Int(ORD(-s), 14); Out.Ln;
  Out.Int(ORD(Full - Top), 14);   Out.Int(ORD(s - {31}), 14); Out.Ln
END Sets.
