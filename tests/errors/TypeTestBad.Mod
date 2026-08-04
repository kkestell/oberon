MODULE TypeTestBad;
  TYPE Root = RECORD value: INTEGER END;
       Child = RECORD (Root) child: INTEGER END;
       Sibling = RECORD (Root) sibling: INTEGER END;
       Other = RECORD other: INTEGER END;
       PRoot = POINTER TO Root;
       PChild = POINTER TO Child;
       PSibling = POINTER TO Sibling;
       POther = POINTER TO Other;
  VAR root: Root; pointer: PRoot; child: PChild; n: INTEGER; target: PChild; b: BOOLEAN;

  PROCEDURE Value(value: Root);
  BEGIN b := value IS Child END Value;

  PROCEDURE RecordTarget(VAR value: Root);
  BEGIN b := value IS PChild END RecordTarget;

  PROCEDURE P; END P;
BEGIN
  b := root IS Child;
  b := n IS PChild;
  b := P IS PChild;
  b := pointer IS target;
  b := pointer IS Child;
  b := child IS Root;
  b := pointer IS POther;
  b := child IS PRoot;
  b := NIL IS Child
END TypeTestBad.
