MODULE InclRange;
  IMPORT Out;
  VAR n: INTEGER; s: SET;
BEGIN
  s := {0}; n := 64; INCL(s, n);
  Out.Int(ORD(s), 4)
END InclRange.
