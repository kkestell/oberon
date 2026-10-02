MODULE RealPacking;
  (* Report 10.2 PACK and UNPK. PACK(x, n) is x := x * 2^n. UNPK is its
     inverse: it preserves the sign of x, normalizes the magnitude into
     1 <= x < 2, and stores the base-two exponent in n.

     Zero satisfies no normalization interval, so this compiler defines UNPK
     of either zero as positive zero with a zero exponent. Dividing into the
     result tells the two zeros apart, because 1.0 / +0.0 is a positive
     infinity. The overflow, subnormal, and underflow rows are ordinary
     binary64 results and not traps. *)
  IMPORT Out;
  VAR x: REAL; n, calls: INTEGER;

  PROCEDURE Exponent(): INTEGER;
  BEGIN
    INC(calls)
  RETURN 3
  END Exponent;

  (* The value parameter is a writable copy, so UNPK can normalize it in
     place. The fraction is scaled by four so a half is visible through
     FLOOR. *)
  PROCEDURE Split(value: REAL);
    VAR e: INTEGER;
  BEGIN
    UNPK(value, e);
    Out.Int(FLOOR(value * 4.), 8); Out.Int(e, 8)
  END Split;

BEGIN
  (* A positive and a negative exponent. 1.5 * 2^4 is 24 and 1.5 * 2^-2 is
     0.375, which is 3 once scaled by eight. *)
  x := 1.5; PACK(x, 4); Out.Int(FLOOR(x), 8); Out.Ln;
  x := 1.5; PACK(x, -2); Out.Int(FLOOR(x * 8.), 8); Out.Ln;

  (* 2^2000 overflows to an infinity, 2^-2000 underflows to zero, and
     2^-1050 lands between the smallest subnormal and the smallest normal
     value. *)
  x := 1.; PACK(x, 2000);  Out.Int(ORD(x > 1.0E308), 8);
  x := 1.; PACK(x, -2000); Out.Int(ORD(x = 0.), 8);
  x := 1.; PACK(x, -1050); Out.Int(ORD(x > 0.), 8); Out.Ln;

  (* An INTEGER exponent is wider than the int accepted by C ldexp. Values
     past that boundary still overflow or underflow instead of wrapping. *)
  x := 1.; PACK(x, 2147483648);  Out.Int(ORD(x > 1.0E308), 8);
  x := 1.; PACK(x, -2147483649); Out.Int(ORD(x = 0.), 8); Out.Ln;

  (* UNPK on a positive, a negative, and a subnormal input. *)
  Split(12.); Out.Ln;
  Split(-12.); Out.Ln;
  x := 1.; PACK(x, -1050); Split(x); Out.Ln;

  (* Either zero normalizes to a positive zero and a zero exponent. *)
  x := 0.;  UNPK(x, n); Out.Int(ORD(1.0 / x > 0.), 8); Out.Int(n, 8); Out.Ln;
  x := -0.; UNPK(x, n); Out.Int(ORD(1.0 / x > 0.), 8); Out.Int(n, 8); Out.Ln;

  (* The exponent expression is evaluated once, after the target address. *)
  calls := 0; x := 1.; PACK(x, Exponent());
  Out.Int(FLOOR(x), 8); Out.Int(calls, 8); Out.Ln
END RealPacking.
