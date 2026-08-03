MODULE RecordParamBad;
  TYPE
    Box = RECORD n: INTEGER; row: ARRAY 2 OF INTEGER END;
    OtherBox = RECORD n: INTEGER; row: ARRAY 2 OF INTEGER END;
    Row = ARRAY 2 OF INTEGER;
    OtherRow = ARRAY 2 OF INTEGER;
    Text = ARRAY 4 OF CHAR;
  VAR box: Box; otherBox: OtherBox; row: Row; otherRow: OtherRow;

  PROCEDURE ChangeOnly(VAR value: Box);
  END ChangeOnly;

  PROCEDURE MutateValue(value: Box; VAR changed: Box);
  BEGIN
    value := changed;
    value.n := 1;
    value.row[0] := 2;
    INC(value.n);
    ChangeOnly(value);
    changed := value
  END MutateValue;

  PROCEDURE TakeBox(value: Box; VAR changed: Box);
  END TakeBox;

  PROCEDURE TakeRow(VAR value: Row);
  END TakeRow;

  PROCEDURE TakeText(value: Text);
  END TakeText;

BEGIN
  MutateValue(box, box);
  TakeBox(otherBox, otherBox);
  TakeRow(otherRow);
  TakeText("longer");
  TakeText("A")
END RecordParamBad.
