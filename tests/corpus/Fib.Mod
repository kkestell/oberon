MODULE Fib;
IMPORT Out;

PROCEDURE Fib(n: INTEGER): INTEGER;
VAR result: INTEGER;
BEGIN
  IF n < 2 THEN
    result := n
  ELSE
    result := Fib(n - 1) + Fib(n - 2)
  END
RETURN result
END Fib;

BEGIN
  Out.Int(Fib(10), 0); Out.Ln
END Fib.
