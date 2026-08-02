MODULE DivMod;
  (* Report 8.2.2: 0 <= x MOD y < y, so DIV floors instead of truncating.
     Every quotient and remainder below is computed twice, once folded by sema
     and once at runtime. The two output lines must be identical. *)
  IMPORT Out;
  CONST
    QA = (-7) DIV 2; RA = (-7) MOD 2;
    QB = 7 DIV 2;    RB = 7 MOD 2;
    QC = (-6) DIV 3; RC = (-6) MOD 3;
  VAR x, y: INTEGER;
BEGIN
  Out.Int(QA, 4); Out.Int(RA, 4);
  Out.Int(QB, 4); Out.Int(RB, 4);
  Out.Int(QC, 4); Out.Int(RC, 4); Out.Ln;

  y := 2; x := -7;
  Out.Int(x DIV y, 4); Out.Int(x MOD y, 4);
  x := 7;
  Out.Int(x DIV y, 4); Out.Int(x MOD y, 4);
  y := 3; x := -6;
  Out.Int(x DIV y, 4); Out.Int(x MOD y, 4); Out.Ln
END DivMod.
