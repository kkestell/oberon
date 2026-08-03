MODULE SetBuiltinBad;
  (* Report 10.2: INCL and EXCL are proper procedures taking a writable SET
     variable and an INTEGER, and ORD takes a BOOLEAN or a SET. *)
  CONST C = {0};
  VAR s: SET; n: INTEGER; b: BOOLEAN;
BEGIN
  INCL(n, 0);
  EXCL(b, 0);
  INCL(C, 0);
  EXCL(s + s, 0);
  INCL(s, b);
  EXCL(s);
  INCL(s, 0, 1);
  n := INCL(s, 0);
  n := EXCL(s, 0);
  n := ORD(n)
END SetBuiltinBad.
