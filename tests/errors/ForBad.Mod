MODULE ForBad;
  (* Every way a FOR statement can be wrong, in one module. The last three
     are the three paths that write through a name: assignment, INC, and a
     VAR actual. All three see the control variable as read-only inside the
     body, following ORP and OJP, which both set obj.rdo around it. *)
  CONST C = 1;
  VAR i: INTEGER; b: BOOLEAN;

  PROCEDURE Step(): INTEGER;
  BEGIN
  RETURN 2
  END Step;

  PROCEDURE Take(VAR v: INTEGER);
  BEGIN v := 0
  END Take;

BEGIN
  FOR b := 1 TO 2 DO END;
  FOR C := 1 TO 2 DO END;
  FOR i := 1 TO b DO END;
  FOR i := 1 TO 2 BY Step() DO END;
  FOR i := 1 TO 2 BY 0 DO END;
  FOR i := 1 TO 2 DO i := 5 END;
  FOR i := 1 TO 2 DO INC(i) END;
  FOR i := 1 TO 2 DO Take(i) END
END ForBad.
