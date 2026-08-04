MODULE PointerTypeGuardFail;
  TYPE Root = RECORD value: INTEGER END;
       Child = RECORD (Root) child: INTEGER END;
       Sibling = RECORD (Root) sibling: INTEGER END;
       PRoot = POINTER TO Root;
       PChild = POINTER TO Child;
       PSibling = POINTER TO Sibling;
  VAR root: PRoot; sibling: PSibling;
BEGIN
  NEW(sibling); root := sibling; root(PChild).child := 1
END PointerTypeGuardFail.
