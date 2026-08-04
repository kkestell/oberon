MODULE NilTypeGuardFail;
  TYPE Root = RECORD value: INTEGER END;
       Child = RECORD (Root) child: INTEGER END;
       PRoot = POINTER TO Root;
       PChild = POINTER TO Child;
  VAR root: PRoot;
BEGIN
  root := NIL; root(PChild).child := 1
END NilTypeGuardFail.
