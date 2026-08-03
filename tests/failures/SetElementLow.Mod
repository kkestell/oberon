MODULE SetElementLow;
  (* A dynamic element below the SET domain traps rather than shifting by a
     count QBE would reduce modulo the word width. *)
  IMPORT Out;
  VAR n: INTEGER; s: SET;
BEGIN
  n := -1; s := {n};
  Out.Int(ORD(s), 4)
END SetElementLow.
