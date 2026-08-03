MODULE FloorConstRange;
  (* FLOOR has a result only where one exists in INTEGER. The upper endpoint
     is reached through FLT rather than through a literal, because
     MAX(INTEGER) rounds up to 2147483648.0 in binary32 and so lands one
     value outside the domain. An infinity and a NaN are rejected on
     finiteness: no comparison a NaN takes part in would reject it. *)
  CONST
    Below = FLOOR(-3.0E9);
    Above = FLOOR(3.0E9);
    Rounded = FLOOR(FLT(2147483647));
    Infinite = FLOOR(1.0 / 0.0);
    NotANumber = FLOOR(0.0 / 0.0);
  VAR n: INTEGER;
BEGIN
  n := FLOOR(3.0E9)
END FloorConstRange.
