MODULE NilFunctionCall;
TYPE Function = PROCEDURE(): INTEGER;
VAR function: Function;
  value: INTEGER;
BEGIN function := NIL; value := function()
END NilFunctionCall.
