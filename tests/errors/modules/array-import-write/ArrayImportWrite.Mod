MODULE ArrayImportWrite;
  (* Report 9.1 and 11: an imported variable is read-only, and selecting an
     element or a row of it does not make that part writable. *)
  IMPORT ArraySource;
  VAR mine: ArraySource.Row;
BEGIN
  ArraySource.data := mine;
  ArraySource.data[0] := 1;
  ArraySource.grid[0] := mine;
  INC(ArraySource.data[1])
END ArrayImportWrite.
