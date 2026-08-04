MODULE ProcedureConstBad;
TYPE Action = PROCEDURE;
  Function = PROCEDURE(): INTEGER;
VAR action: Action;
  function: Function;

PROCEDURE Global;
END Global;

PROCEDURE Check;
CONST Value = Global;
  Relation = action = NIL;
  Activation = function();
END Check;

END ProcedureConstBad.
