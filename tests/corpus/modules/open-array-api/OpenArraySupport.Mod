MODULE OpenArraySupport;
TYPE Item* = RECORD n*: INTEGER END;

PROCEDURE Sum*(a: ARRAY OF INTEGER): INTEGER;
VAR i, total: INTEGER;
BEGIN i := 0; total := 0; WHILE i < LEN(a) DO total := total + a[i]; INC(i) END
RETURN total
END Sum;

PROCEDURE Forward(a: ARRAY OF INTEGER): INTEGER;
BEGIN RETURN Sum(a) END Forward;

PROCEDURE Forwarded*(a: ARRAY OF INTEGER): INTEGER;
BEGIN RETURN Forward(a) END Forwarded;

PROCEDURE Touch*(VAR a: ARRAY OF INTEGER);
BEGIN a[LEN(a) - 1] := 77
END Touch;

PROCEDURE Matrix*(a: ARRAY OF ARRAY OF INTEGER): INTEGER;
BEGIN RETURN a[LEN(a) - 1, LEN(a[0]) - 1] END Matrix;

PROCEDURE TextLength*(a: ARRAY OF CHAR): INTEGER;
BEGIN RETURN LEN(a) END TextLength;

PROCEDURE ItemSum*(a: ARRAY OF Item): INTEGER;
VAR i, total: INTEGER;
BEGIN i := 0; total := 0; WHILE i < LEN(a) DO total := total + a[i].n; INC(i) END
RETURN total
END ItemSum;
END OpenArraySupport.
