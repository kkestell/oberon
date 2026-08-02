MODULE AbsOverflow;
  (* MIN(INTEGER) has no absolute value. Folding rejects it with a
     diagnostic, so the runtime form traps rather than wrapping. *)
  IMPORT Out;
  VAR n: INTEGER;
BEGIN
  n := -2147483648;
  Out.Int(ABS(n), 4)
END AbsOverflow.
