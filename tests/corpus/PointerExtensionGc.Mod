MODULE PointerExtensionGc;
  IMPORT Out;
  TYPE Root = RECORD value: INTEGER END;
       PRoot = POINTER TO Root;
       Child = RECORD (Root) next: PRoot END;
       AtomicChild = RECORD (Root) extra: INTEGER END;
       Empty = RECORD END;
       EmptyChild = RECORD (Empty) END;
       PChild = POINTER TO Child;
       PAtomicChild = POINTER TO AtomicChild;
       PEmpty = POINTER TO Empty;
       PEmptyChild = POINTER TO EmptyChild;
       ScannedBase = RECORD saved: PRoot END;
       ScannedChild = RECORD (ScannedBase) extra: INTEGER END;
       PScannedChild = POINTER TO ScannedChild;
       Holder = RECORD saved: PRoot END;
       PHolder = POINTER TO Holder;
  VAR holder: PHolder; child, temporary: PChild; atomic: PAtomicChild; scanned: PScannedChild;
      empty: PEmpty; emptyChild: PEmptyChild; i: INTEGER;

  PROCEDURE Check(VAR value: Root);
  BEGIN Out.Int(ORD(value IS Child), 0); Out.Int(value(Child).value, 0) END Check;

BEGIN
  NEW(holder); NEW(child); child.value := 77; child.next := NIL; holder.saved := child;
  NEW(scanned); scanned.saved := holder.saved; scanned.extra := 3;
  holder.saved := NIL; child := NIL;
  FOR i := 1 TO 200000 DO NEW(temporary); temporary.value := i; temporary.next := NIL END;
  temporary := NIL;
  Out.Int(ORD(scanned.saved IS PChild), 0); Check(scanned.saved^); Out.Ln;
  NEW(atomic); atomic.value := 8; atomic.extra := 9; Out.Int(ORD(atomic IS PAtomicChild), 0); Out.Int(atomic.extra, 0); Out.Ln;
  NEW(emptyChild); empty := emptyChild; NEW(emptyChild);
  Out.Int(ORD(empty # NIL), 0); Out.Int(ORD(emptyChild # NIL), 0); Out.Int(ORD(empty # emptyChild), 0); Out.Int(ORD(empty IS PEmptyChild), 0); Out.Ln
END PointerExtensionGc.
