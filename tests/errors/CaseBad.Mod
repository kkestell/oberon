MODULE CaseBad;
  (* The reversed range and both overlaps follow oberonc, which keeps one
     ordered label table for the whole statement rather than one per
     alternative. *)
  CONST Yes = TRUE;
  VAR k, v: INTEGER; b: BOOLEAN;
BEGIN
  CASE b OF 1: END;
  CASE k OF v: END;
  CASE k OF Yes: END;
  CASE k OF 5..1: END;
  CASE k OF 1..4: | 3..6: END;
  CASE k OF 7: | 8: | 7: END
END CaseBad.
