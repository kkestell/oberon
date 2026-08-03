MODULE SetInRange;
  (* Membership takes the same domain check as a constructor element. *)
  IMPORT Out;
  VAR n: INTEGER; s: SET;
BEGIN
  s := {0}; n := 32;
  Out.Int(ORD(n IN s), 4)
END SetInRange.
