MODULE ProcedureSupport;
IMPORT Out;
TYPE Action* = PROCEDURE(ch: CHAR);
  GridCallback* = PROCEDURE(a: ARRAY OF ARRAY OF INTEGER);
VAR Current*: Action;

PROCEDURE Echo*(ch: CHAR);
BEGIN Out.Char(ch)
END Echo;

PROCEDURE Shape*(a: ARRAY OF ARRAY OF INTEGER);
BEGIN Out.Int(LEN(a), 1); Out.Char("x"); Out.Int(LEN(a[0]), 1)
END Shape;

PROCEDURE Apply*(action: Action; ch: CHAR);
BEGIN action(ch)
END Apply;

PROCEDURE ApplyGrid*(callback: GridCallback; a: ARRAY OF ARRAY OF INTEGER);
BEGIN callback(a)
END ApplyGrid;

PROCEDURE Get*(): Action;
BEGIN RETURN Echo
END Get;

BEGIN Current := Echo
END ProcedureSupport.
