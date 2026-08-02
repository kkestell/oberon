MODULE CaseNoMatch;
  (* Oberon-07 has no ELSE in a case statement, and Report 9.5 is silent
     about a selector that matches no label. oberonc traps and OBNC raises;
     this compiler traps too. *)
  VAR k: INTEGER;
BEGIN
  k := 9;
  CASE k OF
    1: k := 0
  | 2..4: k := 1
  END
END CaseNoMatch.
