MODULE PointerLayout;
IMPORT Out;

TYPE
  ItemPointer = POINTER TO Item;
  Item = RECORD value: INTEGER END;
  LayoutPointer = POINTER TO Layout;
  Layout = RECORD
    ch: CHAR;
    first: ItemPointer;
    byte: BYTE;
    items: ARRAY 3 OF ItemPointer
  END;

VAR layout: LayoutPointer;

BEGIN
  NEW(layout);
  layout.ch := "A"; layout.byte := 255;
  NEW(layout.first); layout.first.value := 7;
  NEW(layout.items[0]); layout.items[0].value := 8;
  NEW(layout.items[1]); layout.items[1].value := 9;
  NEW(layout.items[2]); layout.items[2].value := 10;
  Out.Char(layout.ch); Out.Int(layout.byte, 0); Out.Int(layout.first.value, 0);
  Out.Int(layout.items[0].value, 0); Out.Int(layout.items[1].value, 0);
  Out.Int(layout.items[2].value, 0); Out.Ln
END PointerLayout.
