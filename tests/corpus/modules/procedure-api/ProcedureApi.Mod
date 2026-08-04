MODULE ProcedureApi;
IMPORT ProcedureSupport, Out;
TYPE OtherAction = PROCEDURE(ch: CHAR);
  Grid = ARRAY 2, 4 OF INTEGER;
VAR action: ProcedureSupport.Action;
  other: OtherAction;
  grid: Grid;
BEGIN
  action := ProcedureSupport.Echo; action("a");
  other := ProcedureSupport.Echo; other("b");
  ProcedureSupport.Current("c");
  ProcedureSupport.Apply(ProcedureSupport.Echo, "d");
  action := ProcedureSupport.Get(); action("e");
  action := Out.Char; action("f");
  ProcedureSupport.ApplyGrid(ProcedureSupport.Shape, grid); Out.Ln
END ProcedureApi.
