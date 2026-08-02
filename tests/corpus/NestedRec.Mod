MODULE NestedRec;
IMPORT Out;
VAR direct, mutual, sibling: INTEGER;

PROCEDURE P;
  PROCEDURE Count(n: INTEGER);
  BEGIN
    direct := direct + 1;
    IF n > 0 THEN Count(n - 1) END
  END Count;
  PROCEDURE Q(n: INTEGER);
    PROCEDURE R(n: INTEGER);
    BEGIN
      mutual := mutual + 10;
      IF n > 0 THEN Q(n - 1) END
    END R;
  BEGIN
    mutual := mutual + 1;
    IF n > 0 THEN R(n) END
  END Q;
  PROCEDURE Earlier(value: INTEGER);
  BEGIN
    sibling := sibling + value
  END Earlier;
  PROCEDURE Later;
  BEGIN
    Earlier(7)
  END Later;
BEGIN
  Count(3);
  Q(2);
  Later
END P;

BEGIN
  direct := 0; mutual := 0; sibling := 0;
  P;
  Out.Int(direct, 0); Out.Int(mutual, 3); Out.Int(sibling, 3); Out.Ln
END NestedRec.
