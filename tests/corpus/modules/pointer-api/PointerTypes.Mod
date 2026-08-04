MODULE PointerTypes;

CONST None* = NIL;

TYPE
  Pointer* = POINTER TO Record;
  Record* = RECORD value*: INTEGER END;
  HiddenPointer* = POINTER TO Hidden;
  Hidden = RECORD public*: INTEGER; private: Pointer END;

VAR root*: Pointer;

PROCEDURE Identity*(p: Pointer): Pointer;
BEGIN
RETURN p
END Identity;

PROCEDURE Clear*(VAR p: Pointer);
BEGIN
  p := NIL
END Clear;

BEGIN
  NEW(root); root.value := 17
END PointerTypes.
