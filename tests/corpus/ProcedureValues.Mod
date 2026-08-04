MODULE ProcedureValues;
IMPORT Out;
TYPE Action = PROCEDURE(ch: CHAR);
  SameAction = PROCEDURE(value: CHAR);
  Number = PROCEDURE(x: INTEGER): INTEGER;
  Nullary = PROCEDURE(): INTEGER;
VAR global, p: Action;
  q: SameAction;
  number: Number;
  nullary: Nullary;

PROCEDURE A(ch: CHAR);
BEGIN Out.Char(ch)
END A;

PROCEDURE B(ch: CHAR);
BEGIN Out.Char(ch)
END B;

PROCEDURE Twice(x: INTEGER): INTEGER;
BEGIN RETURN x * 2
END Twice;

PROCEDURE FortyTwo(): INTEGER;
BEGIN RETURN 42
END FortyTwo;

PROCEDURE Use(p: Action);
BEGIN
  IF p = NIL THEN Out.Char("n") ELSE p("x") END;
  p := B;
  p("b")
END Use;

PROCEDURE Replace(VAR p: Action);
BEGIN p := A
END Replace;

PROCEDURE Pick(): Action;
BEGIN RETURN B
END Pick;

PROCEDURE NoAction(): Action;
BEGIN RETURN NIL
END NoAction;

BEGIN
  p := NIL;
  Use(p);
  p := A;
  q := p;
  q("q");
  Use(A);
  Replace(p);
  p("r");
  number := Twice;
  Out.Int(number(21), 1); Out.Ln;
  nullary := FortyTwo;
  Out.Int(nullary(), 1); Out.Ln;
  p := Pick(); p("p");
  p := NoAction();
  IF (p = NIL) & (A # NIL) & (A = A) & (A # B) THEN Out.Char("e") END;
  global := A;
  global("g"); Out.Ln
END ProcedureValues.
