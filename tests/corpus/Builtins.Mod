MODULE Builtins;
  (* Report 10.2, for the operations whose argument types exist. Every result
     is computed twice: once folded into a constant declaration and once at
     run time from a variable holding the same operand. The two columns of
     each row must be identical, as in DivMod.Mod. *)
  IMPORT Out;
  CONST
    Min = -9223372036854775808;
    AbsNeg = ABS(-9); AbsPos = ABS(9); AbsMax = ABS(Min + 1);
    OddNeg = ODD(-3); OddEven = ODD(4);
    Lsl0 = LSL(5, 0); Lsl63 = LSL(1, 63);
    Asr1 = ASR(-8, 1); Asr63 = ASR(Min, 63);
    Ror0 = ROR(5, 0); Ror1 = ROR(1, 1); Ror63 = ROR(1, 63);
    OrdTrue = ORD(TRUE); OrdFalse = ORD(FALSE);
  VAR n, count: INTEGER; b: BOOLEAN;

  (* INC and DEC on a local, on a value parameter, and on a VAR parameter. *)
  PROCEDURE Bump(v: INTEGER; VAR w: INTEGER);
    VAR local: INTEGER;
  BEGIN
    local := 100; INC(local); DEC(local, 5); Out.Int(local, 6);
    INC(v, 3); Out.Int(v, 6);
    DEC(w)
  END Bump;

BEGIN
  n := -9; Out.Int(AbsNeg, 6); Out.Int(ABS(n), 6);
  n := 9;  Out.Int(AbsPos, 6); Out.Int(ABS(n), 6); Out.Ln;
  n := Min + 1; Out.Int(AbsMax, 21); Out.Int(ABS(n), 21); Out.Ln;

  (* -3 REM 2 is -1, so ODD compares the remainder against zero. Comparing
     it against one would call every negative odd number even. *)
  n := -3; Out.Int(ORD(OddNeg), 6); Out.Int(ORD(ODD(n)), 6);
  n := 4;  Out.Int(ORD(OddEven), 6); Out.Int(ORD(ODD(n)), 6); Out.Ln;

  n := 5; Out.Int(Lsl0, 21);  Out.Int(LSL(n, 0), 21);
  n := 1; Out.Int(Lsl63, 21); Out.Int(LSL(n, 63), 21); Out.Ln;
  n := -8;  Out.Int(Asr1, 21);  Out.Int(ASR(n, 1), 21);
  n := Min; Out.Int(Asr63, 21); Out.Int(ASR(n, 63), 21); Out.Ln;
  n := 5; Out.Int(Ror0, 21);  Out.Int(ROR(n, 0), 21);
  n := 1; Out.Int(Ror1, 21);  Out.Int(ROR(n, 1), 21);
  n := 1; Out.Int(Ror63, 21); Out.Int(ROR(n, 63), 21); Out.Ln;

  (* A count that is not an immediate takes the runtime range check. *)
  n := 1; count := 63; Out.Int(LSL(n, count), 21); Out.Ln;

  b := TRUE;  Out.Int(OrdTrue, 4);  Out.Int(ORD(b), 4);
  b := FALSE; Out.Int(OrdFalse, 4); Out.Int(ORD(b), 4); Out.Ln;

  n := 10; INC(n); Out.Int(n, 6); DEC(n, 4); Out.Int(n, 6);
  count := 50; Bump(7, count); Out.Int(count, 6); Out.Ln;

  ASSERT(n = 7)
END Builtins.
