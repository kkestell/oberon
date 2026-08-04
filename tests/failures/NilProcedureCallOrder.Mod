MODULE NilProcedureCallOrder;
IMPORT Out;
TYPE Action = PROCEDURE(value: INTEGER);
VAR action: Action;

PROCEDURE Actual(): INTEGER;
BEGIN Out.Char("a"); RETURN 1
END Actual;

BEGIN action := NIL; action(Actual())
END NilProcedureCallOrder.
