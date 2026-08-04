MODULE StringsNegativeCount;
IMPORT Strings;
VAR s: ARRAY 2 OF CHAR;
BEGIN
  s := "";
  Strings.Delete(s, 0, -1)
END StringsNegativeCount.
