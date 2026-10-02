MODULE FloorConstRange;
  (* FLOOR has a result only where one exists in INTEGER. The upper endpoint
     is reached through FLT rather than through a literal, because
     MAX(INTEGER) rounds up to 9223372036854775808.0 in binary64 and so lands one
     value outside the domain. An infinity and a NaN are rejected on
     finiteness: no comparison a NaN takes part in would reject it. *)
  CONST
    Below = FLOOR(-1.0E19);
    Above = FLOOR(1.0E19);
    Rounded = FLOOR(FLT(9223372036854775807));
    Infinite = FLOOR(1.0 / 0.0);
    NotANumber = FLOOR(0.0 / 0.0);
  VAR n: INTEGER;
BEGIN
  n := FLOOR(1.0E19)
END FloorConstRange.
