MODULE IntegerLiteralRange;
  (* A decimal literal is a magnitude, so one past MAX(INTEGER) is out of
     range even though its negation would be MIN(INTEGER). *)
  VAR n: INTEGER;
BEGIN
  n := -9223372036854775808;
  n := 9223372036854775808
END IntegerLiteralRange.
