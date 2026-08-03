MODULE CharImportWrite;
  (* Report 9.1 and 11: an imported character array, one of its elements, and
     an imported CHAR variable all reject assignment. *)
  IMPORT CharSource;
BEGIN
  CharSource.text := "abc";
  CharSource.text[0] := "x";
  CharSource.ch := "x"
END CharImportWrite.
