MODULE SetElementHigh;
  IMPORT Out;
  VAR n: INTEGER; s: SET;
BEGIN
  n := 32; s := {n};
  Out.Int(ORD(s), 4)
END SetElementHigh.
