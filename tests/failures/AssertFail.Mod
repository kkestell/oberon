MODULE AssertFail;
  (* The condition is computed at run time, so nothing folds the test away. *)
  VAR n: INTEGER;
BEGIN
  n := 1;
  ASSERT(n = 2)
END AssertFail.
