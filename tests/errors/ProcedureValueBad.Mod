MODULE ProcedureValueBad;
TYPE Action = PROCEDURE;
  OtherAction = PROCEDURE;
VAR action: Action;
  other: OtherAction;

PROCEDURE Global;
END Global;

PROCEDURE Take(action: Action);
END Take;

PROCEDURE Replace(VAR action: Action);
BEGIN action := Global
END Replace;

PROCEDURE Outer;
  PROCEDURE Local;
  END Local;
  PROCEDURE Bad(): Action;
  BEGIN RETURN Local
  END Bad;
BEGIN
  action := Local;
  Take(Local);
  IF Local = NIL THEN action := Global END;
  action := Bad()
END Outer;

BEGIN
  action := ABS;
  Take(INC);
  IF LEN = NIL THEN action := Global END;
  Replace(Global);
  Replace(NIL);
  Replace(other)
END ProcedureValueBad.
