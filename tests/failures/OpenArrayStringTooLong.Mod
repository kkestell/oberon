MODULE OpenArrayStringTooLong;
VAR text: ARRAY 3 OF CHAR;
PROCEDURE Copy(VAR a: ARRAY OF CHAR);
BEGIN a := "abc"
END Copy;
BEGIN text := "x"; Copy(text)
END OpenArrayStringTooLong.
