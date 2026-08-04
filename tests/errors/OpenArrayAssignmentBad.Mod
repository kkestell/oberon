MODULE OpenArrayAssignmentBad;
VAR fixed: ARRAY 3 OF INTEGER;
    fixedReal: ARRAY 3 OF REAL;
    fixedGrid: ARRAY 2, 2 OF INTEGER;
    chars: ARRAY 4 OF CHAR;

PROCEDURE Bad(VAR destination: ARRAY OF INTEGER; source: ARRAY OF INTEGER;
              matrix: ARRAY OF ARRAY OF INTEGER; text: ARRAY OF CHAR);
BEGIN
  destination := fixed;
  destination := source;
  fixedReal := source;
  fixedGrid := matrix;
  text := "x"
END Bad;
BEGIN Bad(fixed, fixed, fixedGrid, chars)
END OpenArrayAssignmentBad.
