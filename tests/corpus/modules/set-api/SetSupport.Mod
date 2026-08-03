MODULE SetSupport;
  (* SET in every exported form: a constant, a variable initialized by the
     module body, a procedure taking value and VAR SET parameters, and a
     function returning SET. *)
  CONST Mask* = {1, 3, 5};
  VAR flags*: SET;

  PROCEDURE Merge*(extra: SET; VAR into: SET);
  BEGIN
    into := into + extra;
    (* The copy is writable and the caller never sees this. *)
    INCL(extra, 31)
  END Merge;

  PROCEDURE Range*(low, high: INTEGER): SET;
  BEGIN
  RETURN {low .. high}
  END Range;

BEGIN
  flags := {0, 2}
END SetSupport.
