MODULE Chars;
  (* Report 6.1 and 8.2.4. Every relation is computed twice: once folded into
     a constant declaration and once at run time from variables holding the
     same operands, and the two columns of each row must match, as in
     Sets.Mod. The quoted and ordinal literal forms are both strings, so "A"
     and 41X are interchangeable, and the boundary ordinals 0 and 255 are
     ordinary characters. *)
  IMPORT Out;
  CONST
    A = "A";
    Z = 5AX;
    Nul = 0X;
    Top = 0FFX;
    OrdA = ORD(A);
    OrdZ = ORD(Z);
    OrdNul = ORD(Nul);
    OrdTop = ORD(Top);
    Back = CHR(OrdA);
    Eq = A = Back;
    Ne = A # Z;
    Lt = A < Z;
    Le = Z <= A;
    Gt = Top > Nul;
    Ge = Nul >= Top;
  VAR ch, top: CHAR; n: INTEGER;

  PROCEDURE Pass(c: CHAR): CHAR;
    VAR t: CHAR;
  BEGIN
    t := c
  RETURN t
  END Pass;

  PROCEDURE Put(VAR c: CHAR; v: CHAR);
  BEGIN
    c := v
  END Put;

BEGIN
  ch := "A"; top := 0FFX;
  (* ORD and CHR in both directions, folded and computed *)
  Out.Int(OrdA, 6);   Out.Int(ORD(ch), 6); Out.Ln;
  Out.Int(OrdZ, 6);   n := 90; Out.Int(ORD(CHR(n)), 6); Out.Ln;
  Out.Int(OrdNul, 6); Put(ch, 0X); Out.Int(ORD(ch), 6); Out.Ln;
  Out.Int(OrdTop, 6); Out.Int(ORD(top), 6); Out.Ln;
  ch := Pass(Back);
  Out.Char(ch); Out.Char(Pass("B")); Out.Char(Z); Out.Ln;
  (* all six relations *)
  Out.Int(ORD(Eq), 6); Out.Int(ORD(ch = Pass("A")), 6); Out.Ln;
  Out.Int(ORD(Ne), 6); Out.Int(ORD(ch # Z), 6); Out.Ln;
  Out.Int(ORD(Lt), 6); Out.Int(ORD(ch < Z), 6); Out.Ln;
  Out.Int(ORD(Le), 6); Out.Int(ORD(Z <= ch), 6); Out.Ln;
  Out.Int(ORD(Gt), 6); Out.Int(ORD(top > 0X), 6); Out.Ln;
  Out.Int(ORD(Ge), 6); Out.Int(ORD(0X >= top), 6); Out.Ln
END Chars.
