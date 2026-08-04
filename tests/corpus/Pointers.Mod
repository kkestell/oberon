MODULE Pointers;
IMPORT Out;

CONST None* = NIL; NilFolds = (NIL = NIL);

TYPE
  P* = POINTER TO Node;
  Q = POINTER TO Node;
  Alias = P;
  Node* = RECORD
    value*: INTEGER;
    next*: P;
    rows*: ARRAY 2 OF ARRAY 3 OF INTEGER
  END;
  Holder = RECORD
    selected: P;
    slots: ARRAY 2 OF P
  END;
  HolderPointer = POINTER TO Holder;
  Inline = POINTER TO RECORD value: INTEGER END;
  EmptyPointer = POINTER TO Empty;
  Empty = RECORD END;
  PointerArray = ARRAY 2 OF P;

VAR head*: P;
  other: Q;
  aliased: Alias;
  holder: HolderPointer;
  indexCalls: INTEGER;
  dynamicIndex: INTEGER;
  inline: Inline;
  empty1, empty2: EmptyPointer;
  pointers, copiedPointers: PointerArray;
  record: Node;
  churn: P;
  i: INTEGER;

PROCEDURE Index(): INTEGER;
BEGIN
  INC(indexCalls)
RETURN 1
END Index;

PROCEDURE Same(p: P): P;
BEGIN
RETURN p
END Same;

PROCEDURE Clear(VAR p: P);
BEGIN
  p := NIL
END Clear;

PROCEDURE ValueCopy(p: P);
BEGIN
  p := NIL
END ValueCopy;

PROCEDURE Constants;
CONST A = LEN(head.rows); B = LEN(head^.rows[dynamicIndex]);
BEGIN
  Out.Int(A, 0); Out.Int(B, 0)
END Constants;

PROCEDURE LocalTypes;
TYPE
  OuterBinding = POINTER TO Node;
  Node = RECORD other: INTEGER END;
  Forward = POINTER TO ForwardRecord;
  ForwardRecord = RECORD n: INTEGER END;
VAR outer: OuterBinding; forward: Forward; local: Node;
BEGIN
  NEW(outer); outer.value := 8; Out.Int(outer.value, 0);
  NEW(forward); forward.n := 9;
  local.other := forward.n
END LocalTypes;

PROCEDURE Fill(VAR values: PointerArray);
BEGIN
  NEW(values[0]); values[0].value := 2;
  NEW(values[1]); values[1].value := 3
END Fill;

PROCEDURE Read(values: PointerArray);
BEGIN
  Out.Int(values[0].value, 0); Out.Int(values[1].value, 0)
END Read;

BEGIN
  NEW(head);
  head.value := 41;
  head.next := NIL;
  head.rows[0, 0] := 7;
  other := head;
  Out.Int(other^.value, 0);
  IF other = head THEN Out.Int(1, 0) END;
  IF head # None THEN Out.Int(1, 0) END;
  IF NIL = NIL THEN Out.Int(1, 0) END;
  IF NilFolds THEN Out.Int(1, 0) END;
  ValueCopy(head);
  Out.Int(head.value, 0);
  head := Same(other);
  record := head^;
  Out.Int(record.value, 0);
  record.value := 42;
  head^ := record;
  Out.Int(head^.value, 0);
  NEW(holder);
  NEW(holder.selected);
  holder.selected.value := 5;
  NEW(holder.slots[Index()]);
  holder.slots[1].value := 6;
  Out.Int(holder.selected.value, 0);
  Out.Int(holder.slots[1].value, 0);
  Out.Int(indexCalls, 0);
  dynamicIndex := 0;
  Constants;
  Out.Int(LEN(head.rows), 0);
  Out.Int(LEN(head^.rows[dynamicIndex]), 0);
  aliased := head;
  Out.Int(aliased^.value, 0);
  Clear(aliased);
  IF aliased = NIL THEN Out.Int(1, 0) END;
  Clear(head);
  IF head = NIL THEN Out.Int(1, 0) END;
  NEW(inline); inline.value := 4; Out.Int(inline.value, 0);
  NEW(empty1); NEW(empty2);
  IF (empty1 # NIL) & (empty2 # NIL) THEN Out.Int(1, 0) END;
  IF empty1 # empty2 THEN Out.Int(1, 0) END;
  LocalTypes;
  Fill(pointers); copiedPointers := pointers;
  FOR i := 1 TO 100000 DO NEW(churn) END;
  Read(copiedPointers);
  Out.Ln
END Pointers.
