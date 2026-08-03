MODULE ArrayAssignmentBad;
  (* Report 9.1 requires the same type on both sides, and Report 6.2 makes
     each ARRAY in the source its own type however alike two of them look. *)
  TYPE Row = ARRAY 3 OF INTEGER;
       OtherRow = ARRAY 3 OF INTEGER;
       Grid = ARRAY 2, 3 OF INTEGER;
       OtherGrid = ARRAY 2, 3 OF INTEGER;
  VAR row: Row; otherRow: OtherRow;
      grid: Grid; otherGrid: OtherGrid;
      shorter: ARRAY 2 OF INTEGER;
      reals: ARRAY 3 OF REAL;
      n: INTEGER;
BEGIN
  row := otherRow;
  grid := otherGrid;
  row := shorter;
  row := reals;
  row := n;
  n := row
END ArrayAssignmentBad.
