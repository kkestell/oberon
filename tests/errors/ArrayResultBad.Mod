MODULE ArrayResultBad;
  (* Report 10.1: the result type of a procedure can be neither a record nor
     an array. This one never becomes legal. *)
  TYPE Row = ARRAY 3 OF INTEGER;
  VAR shared: Row;

  PROCEDURE Make(): Row;
  BEGIN
  RETURN shared
  END Make;

END ArrayResultBad.
