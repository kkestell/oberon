MODULE ForNested;
  (* An inner FOR statement over the outer control variable finds it already
     read-only. The assignment after the rejected inner loop proves that the
     recovery restored the outer state rather than clearing it. *)
  VAR i: INTEGER;
BEGIN
  FOR i := 1 TO 2 DO
    FOR i := 1 TO 2 DO END;
    i := 0
  END
END ForNested.
