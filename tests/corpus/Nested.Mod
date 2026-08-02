MODULE Nested;
IMPORT Out;
VAR total, result: INTEGER;

PROCEDURE AddGlobal(value: INTEGER);
BEGIN
  total := total + value
END AddGlobal;

PROCEDURE P;
CONST PBase = 10;
VAR p: INTEGER;
  PROCEDURE Q;
  CONST QBase = 3;
  VAR q: INTEGER;
    PROCEDURE R(value: INTEGER; VAR out: INTEGER);
    BEGIN
      total := total + PBase + QBase;
      AddGlobal(value);
      out := total
    END R;
    PROCEDURE F(value: INTEGER): INTEGER;
    VAR local: INTEGER;
    BEGIN
      local := value + total
    RETURN local
    END F;
  BEGIN
    q := 0;
    R(2, q);
    result := F(q)
  END Q;
BEGIN
  p := 5;
  total := total + p;
  Q;
  total := total + p
END P;

BEGIN
  total := 1;
  result := 0;
  P;
  Out.Int(total, 0); Out.Int(result, 3); Out.Ln
END Nested.
