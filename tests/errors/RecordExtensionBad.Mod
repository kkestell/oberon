MODULE RecordExtensionBad;
  TYPE Missing = RECORD (Absent) END;
       NotRecord = RECORD (INTEGER) END;
       Forward = RECORD (Later) END;
       Later = RECORD END;
       Base = RECORD value: INTEGER END;
       Child = RECORD (Base) value: INTEGER END;
       GoodChild = RECORD (Base) child: INTEGER END;
       Sibling = RECORD (Base) sibling: INTEGER END;
       PBase = POINTER TO Base;
       PChild = POINTER TO GoodChild;
       PSibling = POINTER TO Sibling;
  VAR base: Base; child: GoodChild; sibling: Sibling;
      pbase: PBase; pchild: PChild; psibling: PSibling; equal: BOOLEAN;

  PROCEDURE NeedChild(value: GoodChild); END NeedChild;
  PROCEDURE PointerVar(VAR value: PBase); END PointerVar;
BEGIN
  child := base;
  child := sibling;
  NeedChild(base);
  pchild := pbase;
  psibling := pchild;
  equal := pchild = psibling;
  PointerVar(pchild)
END RecordExtensionBad.
