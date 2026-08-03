MODULE SetBad;
  (* Report 8.2 gives SET four binary operators, complement, equality, and
     inequality, and nothing else. "/" is symmetric difference until REAL
     arrives, so it has no INTEGER meaning yet. *)
  VAR s, t: SET; n: INTEGER; b: BOOLEAN;
BEGIN
  s := s + n;
  n := n * s;
  n := n / n;
  b := s < t;
  b := s >= t;
  s := {TRUE};
  s := {b .. 3};
  s := s DIV t;
  s := s MOD t;
  b := s & t;
  b := ~s;
  s := +s;
  s := n;
  n := s;
  b := s
END SetBad.
