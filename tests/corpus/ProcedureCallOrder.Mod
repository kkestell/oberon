MODULE ProcedureCallOrder;
IMPORT Out;
TYPE Action = PROCEDURE(ch: CHAR);
  Table = ARRAY 1 OF Action;
VAR table: Table;

PROCEDURE A(ch: CHAR);
BEGIN Out.Char("A")
END A;

PROCEDURE B(ch: CHAR);
BEGIN Out.Char("B")
END B;

PROCEDURE TargetIndex(): INTEGER;
BEGIN Out.Char("t"); RETURN 0
END TargetIndex;

PROCEDURE Change(): CHAR;
BEGIN Out.Char("a"); table[0] := B; RETURN "x"
END Change;

BEGIN
  table[0] := A;
  table[TargetIndex()](Change());
  table[0]("x"); Out.Ln
END ProcedureCallOrder.
