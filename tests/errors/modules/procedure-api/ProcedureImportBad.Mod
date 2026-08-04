MODULE ProcedureImportBad;
IMPORT ProcedureSource;
TYPE Bad = ProcedureSource.HiddenAction;
VAR action: ProcedureSource.Action;

PROCEDURE Replace(VAR action: ProcedureSource.Action);
BEGIN action := ProcedureSource.Visible
END Replace;

BEGIN
  action := ProcedureSource.Hidden;
  ProcedureSource.Current := action;
  ProcedureSource.ExportedBox.action := action;
  Replace(ProcedureSource.Current)
END ProcedureImportBad.
