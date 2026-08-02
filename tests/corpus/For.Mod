MODULE For;
  (* Report 9.8. Every loop prints the values its body sees, then the value
     the control variable holds afterwards, which is the first value that
     failed the loop test. *)
  IMPORT Out;
  VAR i, j, sum: INTEGER;
BEGIN
  FOR i := 1 TO 5 DO Out.Int(i, 3) END;
  Out.Int(i, 5); Out.Ln;

  FOR i := 5 TO 1 BY -1 DO Out.Int(i, 3) END;
  Out.Int(i, 5); Out.Ln;

  (* A step that overshoots the limit: 7 is the last value taken. *)
  FOR i := 1 TO 8 BY 3 DO Out.Int(i, 3) END;
  Out.Int(i, 5); Out.Ln;

  (* The first test already fails, so the body never runs and the control
     variable keeps its starting value. *)
  FOR i := 3 TO 2 DO Out.Int(i, 3) END;
  Out.Int(i, 5); Out.Ln;

  FOR i := 1 TO 3 BY -1 DO Out.Int(i, 3) END;
  Out.Int(i, 5); Out.Ln;

  sum := 0;
  FOR i := 1 TO 3 DO
    FOR j := 1 TO 3 DO sum := sum + i * j END
  END;
  Out.Int(sum, 5); Out.Int(i, 3); Out.Int(j, 3); Out.Ln
END For.
