MODULE StringsZeroDestination;
IMPORT Strings;
VAR dest: ARRAY 0 OF CHAR;
BEGIN
  Strings.Extract("a", 0, 1, dest)
END StringsZeroDestination.
