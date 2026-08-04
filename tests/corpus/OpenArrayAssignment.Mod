MODULE OpenArrayAssignment;
IMPORT Out;
TYPE Row = ARRAY 5 OF INTEGER;
     Empty = ARRAY 0 OF INTEGER;
     Item = RECORD n: INTEGER END;
     Items = ARRAY 3 OF Item;
VAR zero: Empty;
    one: ARRAY 1 OF INTEGER;
    three: ARRAY 3 OF INTEGER;
    dst: Row;
    itemSource: ARRAY 2 OF Item;
    itemDest: Items;
    table: ARRAY 2 OF Row;
    text: ARRAY 6 OF CHAR;

PROCEDURE CopyInts(source: ARRAY OF INTEGER; VAR destination: Row);
BEGIN destination := source
END CopyInts;

PROCEDURE CopyItems(source: ARRAY OF Item; VAR destination: Items);
BEGIN destination := source
END CopyItems;

PROCEDURE CopyRow(source: ARRAY OF ARRAY OF INTEGER; i: INTEGER; VAR destination: Row);
BEGIN destination := source[i]
END CopyRow;

PROCEDURE SetText(VAR destination: ARRAY OF CHAR);
BEGIN destination := "fit"
END SetText;

PROCEDURE PrintRow(a: ARRAY OF INTEGER);
VAR i: INTEGER;
BEGIN i := 0; WHILE i < LEN(a) DO Out.Int(a[i], 0); Out.Char(" "); INC(i) END; Out.Ln
END PrintRow;

BEGIN
  dst[0] := 9; dst[1] := 9; dst[2] := 9; dst[3] := 9; dst[4] := 9;
  one[0] := 1; three[0] := 2; three[1] := 3; three[2] := 4;
  CopyInts(zero, dst); PrintRow(dst);
  CopyInts(one, dst); PrintRow(dst);
  CopyInts(three, dst); PrintRow(dst);
  CopyInts(dst, dst); PrintRow(dst);
  itemSource[0].n := 7; itemSource[1].n := 8; itemDest[2].n := 99;
  CopyItems(itemSource, itemDest);
  Out.Int(itemDest[0].n, 0); Out.Char(":"); Out.Int(itemDest[1].n, 0); Out.Char(":"); Out.Int(itemDest[2].n, 0); Out.Ln;
  table[1, 0] := 4; table[1, 1] := 5; table[1, 2] := 6; table[1, 3] := 7; table[1, 4] := 8;
  CopyRow(table, 1, dst); PrintRow(dst);
  SetText(text); IF text = "fit" THEN Out.Int(1, 0) ELSE Out.Int(0, 0) END; Out.Ln
END OpenArrayAssignment.
