MODULE OpenArrayCopyTooLong;
TYPE Small = ARRAY 2 OF INTEGER;
VAR source: ARRAY 3 OF INTEGER;
    destination: Small;
PROCEDURE Copy(a: ARRAY OF INTEGER; VAR b: Small);
BEGIN b := a
END Copy;
BEGIN
  destination[0] := 8; destination[1] := 9;
  Copy(source, destination)
END OpenArrayCopyTooLong.
