MODULE StringsInvalidPosition;
IMPORT Strings;
VAR s: ARRAY 2 OF CHAR;
BEGIN
  s := "";
  Strings.Delete(s, -1, 0)
END StringsInvalidPosition.
