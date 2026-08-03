MODULE RealBad;
  (* Report 8.2: INTEGER and REAL are distinct types and never mix
     implicitly, in an operator, an assignment, an argument, or a result.
     Every conversion has to be written with FLT or FLOOR. "/" has a REAL
     meaning and a SET meaning and no INTEGER one; DIV and MOD are the
     INTEGER operators and have no REAL form. *)
  VAR x, y: REAL; n: INTEGER; s: SET; b: BOOLEAN;

  PROCEDURE Take(v: REAL);
  BEGIN
  END Take;

  PROCEDURE Give(): REAL;
  BEGIN
  RETURN 1
  END Give;

BEGIN
  x := x + n;
  n := n - x;
  x := n * x;
  n := n / n;
  x := x DIV y;
  x := x MOD y;
  b := x & y;
  b := x OR y;
  b := ~x;
  b := x < n;
  b := n >= x;
  b := x = n;
  x := x + s;
  s := s - x;
  b := x = s;
  x := n;
  n := x;
  Take(n);
  Take(s)
END RealBad.
