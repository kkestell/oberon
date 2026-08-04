MODULE ProcedureSource;
TYPE Action* = PROCEDURE;
  HiddenAction = PROCEDURE;
  Box* = RECORD action*: Action END;
VAR Current*: Action;
  ExportedBox*: Box;

PROCEDURE Visible*;
END Visible;

PROCEDURE Hidden;
END Hidden;

BEGIN
  Current := Visible;
  ExportedBox.action := Visible
END ProcedureSource.
