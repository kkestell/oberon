MODULE SetElementConst;
  (* An element the compiler can fold is diagnosed, not deferred to the
     runtime check: writing 32 as 16 + 16 or as a named constant does not
     make it legal. An invalid constant expression is also diagnosed instead
     of being mistaken for a dynamic value. Both endpoints of a range are
     checked before the reversed-range rule chooses the empty set. *)
  CONST Too = 32; Half = 16;
  VAR s: SET;
BEGIN
  s := {-1};
  s := {32};
  s := {Too};
  s := {Half + Half};
  s := {33 .. 40};
  s := {0 .. 32};
  s := {Too .. 0};
  s := {1 DIV 0}
END SetElementConst.
