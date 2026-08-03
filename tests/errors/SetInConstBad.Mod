MODULE SetInConstBad;
  (* The Report requires only an INTEGER left operand for IN. This compiler
     holds it to the same domain as a constructor element, so membership
     never depends on what the backend does with a shift count. *)
  CONST Below = -1; Above = 32; Folded = 32 IN {0};
  VAR s: SET; b: BOOLEAN;
BEGIN
  b := Below IN s;
  b := Above IN s;
  b := (16 + 16) IN s
END SetInConstBad.
