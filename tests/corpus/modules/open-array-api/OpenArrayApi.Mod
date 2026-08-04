MODULE OpenArrayApi;
IMPORT Out, OpenArraySupport;
VAR row: ARRAY 3 OF INTEGER;
    grid: ARRAY 2, 2 OF INTEGER;
    items: ARRAY 2 OF OpenArraySupport.Item;
BEGIN
  row[0] := 1; row[1] := 2; row[2] := 3;
  grid[1, 1] := 44;
  items[0].n := 5; items[1].n := 6;
  Out.Int(OpenArraySupport.Sum(row), 0); Out.Ln;
  Out.Int(OpenArraySupport.Forwarded(grid[0]), 0); Out.Ln;
  OpenArraySupport.Touch(row); Out.Int(row[2], 0); Out.Ln;
  Out.Int(OpenArraySupport.Matrix(grid), 0); Out.Ln;
  Out.Int(OpenArraySupport.TextLength("hello"), 0); Out.Ln;
  Out.Int(OpenArraySupport.ItemSum(items), 0); Out.Ln
END OpenArrayApi.
