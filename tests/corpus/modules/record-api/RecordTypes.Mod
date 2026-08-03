MODULE RecordTypes;
  TYPE
    Public* = RECORD
      shown*: INTEGER; hidden: INTEGER; row*: ARRAY 2 OF INTEGER;
      nested*: RECORD shown*: INTEGER; hidden: INTEGER END
    END;
    Private = RECORD value*: INTEGER; secret: INTEGER END;
  VAR shared*: Public;
      private*: Private;

  PROCEDURE Read*(p: Public): INTEGER;
  RETURN p.shown + p.hidden + p.row[0] + p.row[1]
  END Read;

  PROCEDURE Change*(VAR p: Public; n: INTEGER);
  BEGIN p.shown := n; p.hidden := n + 1; p.row[1] := n + 2 END Change;

BEGIN
  shared.shown := 1; shared.hidden := 2; shared.row[0] := 3; shared.row[1] := 4;
  shared.nested.shown := 5; shared.nested.hidden := 6;
  private.value := 9; private.secret := 10
END RecordTypes.
