MODULE ForLimit;
  (* Report 9.8 rewrites the statement as a WHILE whose condition contains
     the limit expression, so the limit is evaluated before every test,
     including the final test that fails. A loop that runs its body three
     times therefore calls Limit four times, and a loop whose body never runs
     still calls it once. All three vendored compilers agree.

     Limit returns a fixed value and counts its calls in a separate variable.
     A function that returned its own counter would move the limit along with
     the control variable and never terminate. *)
  IMPORT Out;
  VAR calls, i: INTEGER;

  PROCEDURE Limit(): INTEGER;
  BEGIN
    calls := calls + 1
  RETURN 3
  END Limit;

BEGIN
  calls := 0;
  FOR i := 1 TO Limit() DO Out.Int(i, 3) END;
  Out.Int(calls, 5); Out.Ln;

  calls := 0;
  FOR i := 9 TO Limit() DO Out.Int(i, 3) END;
  Out.Int(calls, 5); Out.Ln
END ForLimit.
