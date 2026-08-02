MODULE NotAccessible;
VAR sink: INTEGER;

PROCEDURE Take(VAR value: INTEGER);
BEGIN
  value := value + 1
END Take;

PROCEDURE Outer(param: INTEGER);
VAR local: INTEGER;
  PROCEDURE Read;
  BEGIN
    sink := local
  END Read;
  PROCEDURE Write;
  BEGIN
    local := 1
  END Write;
  PROCEDURE Pass;
  BEGIN
    Take(local)
  END Pass;
  PROCEDURE Middle;
    PROCEDURE Deep;
    BEGIN
      sink := param
    END Deep;
  BEGIN
    Deep
  END Middle;
BEGIN
  Read; Write; Pass; Middle
END Outer;

PROCEDURE ByRef(VAR ref: INTEGER);
  PROCEDURE UseRef;
  BEGIN
    sink := ref
  END UseRef;
BEGIN
  UseRef
END ByRef;

BEGIN
  Outer(0);
  ByRef(sink)
END NotAccessible.
