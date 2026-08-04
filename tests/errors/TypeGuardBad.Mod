MODULE TypeGuardBad;
  TYPE Root = RECORD value: INTEGER END;
       Child = RECORD (Root) child: INTEGER; row: ARRAY 2 OF INTEGER END;
       Other = RECORD other: INTEGER END;
       PRoot = POINTER TO Root;
       PChild = POINTER TO Child;
       POther = POINTER TO Other;
  VAR root: Root; pointer: PRoot; child: PChild; n: INTEGER; target: PChild; b: BOOLEAN;

  PROCEDURE Value(value: Root);
  BEGIN n := value(Child).child END Value;

  PROCEDURE RecordTarget(VAR value: Root);
  BEGIN n := value(PChild).value END RecordTarget;

  PROCEDURE ConstantGuard(VAR value: Root);
    CONST N = LEN(value(Child).row);
  END ConstantGuard;

  PROCEDURE P(x: INTEGER); END P;
BEGIN
  n := root(Child).child;
  b := n(PChild);
  n := pointer(target).value;
  n := pointer(Child).value;
  n := child(PRoot).value;
  n := pointer(POther).other;
  b := pointer(PChild, PRoot);
  b := pointer(n + 1)
END TypeGuardBad.
