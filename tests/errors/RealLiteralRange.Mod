MODULE RealLiteralRange;
  (* Report 3 gives no range to a REAL literal, so the range is this target's
     binary32 representation: a literal whose value rounds to an infinity has
     nothing it could mean. A literal too small to represent rounds to zero
     instead, which is the ordinary IEEE result and not an error. *)
  CONST TooBig = 3.5E38;
  VAR x: REAL;
BEGIN
  x := 1.0E39
END RealLiteralRange.
