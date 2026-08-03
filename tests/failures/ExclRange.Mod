MODULE ExclRange;
  IMPORT Out;
  VAR n: INTEGER; s: SET;
BEGIN
  s := {0}; n := -1; EXCL(s, n);
  Out.Int(ORD(s), 4)
END ExclRange.
