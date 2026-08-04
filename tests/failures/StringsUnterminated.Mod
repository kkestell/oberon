MODULE StringsUnterminated;
IMPORT Strings;
VAR source: ARRAY 2 OF CHAR; dest: ARRAY 4 OF CHAR;
BEGIN
  source[0] := "a"; source[1] := "b"; dest := "";
  Strings.Insert(source, 0, dest)
END StringsUnterminated.
