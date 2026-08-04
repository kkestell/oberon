MODULE OpenArrayInnerIndexHigh;
VAR a: ARRAY 2, 3 OF INTEGER;
PROCEDURE Fail(x: ARRAY OF ARRAY OF INTEGER): INTEGER;
BEGIN RETURN x[1, 3] END Fail;
BEGIN a[0, 0] := Fail(a)
END OpenArrayInnerIndexHigh.
