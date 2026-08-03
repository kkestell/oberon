MODULE StringCompare;
  (* Report 8.2.4 on character arrays and strings, under the bounded rule: the
     walk stops at the first differing pair, at a null present in both, or at
     the shorter operand's length. Each pair prints a folded column and a
     computed one, and the two must agree because the fold applies the same
     rule to the same byte sequences. *)
  IMPORT Out;
  CONST
    Eq = "abc" = "abc";
    Prefix = "ab" < "abc";
    First = "zbc" > "abc";
    Last = "abd" # "abc";
    Le = "abc" <= "abc";
    Ge = "abc" >= "abd";
  VAR a, b: ARRAY 4 OF CHAR;
      full: ARRAY 3 OF CHAR;
BEGIN
  (* equal values: two literals folded, an array against a literal, and an
     array against another array *)
  a := "abc"; b := "abc";
  Out.Int(ORD(Eq), 4); Out.Int(ORD(a = "abc"), 4); Out.Int(ORD(a = b), 4);
  Out.Ln;
  (* a difference in the last position *)
  b := "abd";
  Out.Int(ORD(Last), 4); Out.Int(ORD(a # b), 4); Out.Int(ORD(a # "abd"), 4);
  Out.Ln;
  (* a proper prefix is less: its terminator meets the longer one's next
     character, and the null is below every ordinal *)
  a := "ab";
  Out.Int(ORD(Prefix), 4); Out.Int(ORD(a < b), 4); Out.Int(ORD(a < "abc"), 4);
  Out.Ln;
  (* a difference in the first position *)
  b := "zbc";
  Out.Int(ORD(First), 4); Out.Int(ORD(b > a), 4); Out.Int(ORD(b > "abc"), 4);
  Out.Ln;
  Out.Int(ORD(Le), 4); Out.Int(ORD(b <= b), 4); Out.Ln;
  Out.Int(ORD(Ge), 4); Out.Int(ORD(a >= b), 4); Out.Ln;
  (* an array filled to its last element holds no null character, and the
     comparison still answers without reading past its declared length *)
  full[0] := "a"; full[1] := "b"; full[2] := "c";
  Out.Int(ORD(full = "abc"), 4); Out.Int(ORD(full < "abd"), 4);
  Out.Int(ORD(full > "ab"), 4); Out.Ln;
  (* two values that differ only past the terminator compare equal *)
  a := "ab"; b := "ab";
  a[3] := "x"; b[3] := "y";
  Out.Int(ORD(a = b), 4); Out.Int(ORD(a # b), 4); Out.Ln
END StringCompare.
