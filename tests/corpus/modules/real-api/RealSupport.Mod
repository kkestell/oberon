MODULE RealSupport;
  (* REAL in every exported form: a constant, a variable initialized by the
     module body, a procedure taking value and VAR REAL parameters, and a
     function returning REAL. *)
  CONST Ratio* = 2.5;
  VAR scale*: REAL;

  PROCEDURE Apply*(factor: REAL; VAR into: REAL);
  BEGIN
    into := into * factor;
    (* The copy is writable and the caller never sees this. *)
    factor := 0.
  END Apply;

  PROCEDURE Sum*(a, b: REAL): REAL;
  BEGIN
  RETURN a + b
  END Sum;

BEGIN
  scale := 4.
END RealSupport.
