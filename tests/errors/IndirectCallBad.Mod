MODULE IndirectCallBad;
TYPE Action = PROCEDURE(value: INTEGER);
  Function = PROCEDURE(value: INTEGER): INTEGER;
  Rec = RECORD n: INTEGER END;
VAR action: Action;
  function: Function;
  record: Rec;
  n: INTEGER;
BEGIN
  action();
  action(TRUE);
  function(1);
  n := action(1);
  n := function;
  n := function();
  record.n()
END IndirectCallBad.
