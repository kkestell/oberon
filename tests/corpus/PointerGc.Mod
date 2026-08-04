MODULE PointerGc;
IMPORT Out;

TYPE
  Link = POINTER TO Node;
  Node = RECORD value: INTEGER; next: Link END;
  AtomicPointer = POINTER TO Atomic;
  Atomic = RECORD value: INTEGER END;
  ZeroPointer = POINTER TO Zero;
  Zero = RECORD unused: ARRAY 0 OF Link END;
  Nested = RECORD links: ARRAY 2 OF Link END;
  HolderPointer = POINTER TO Holder;
  Holder = RECORD selected: Link; slots: ARRAY 2 OF Link; nested: Nested END;

VAR head, temporary: Link;
  atomic: AtomicPointer;
  zero: ZeroPointer;
  holder: HolderPointer;
  i, count, sum, indexCalls: INTEGER;

PROCEDURE Index(): INTEGER;
BEGIN
  INC(indexCalls)
RETURN 1
END Index;

BEGIN
  NEW(atomic); atomic.value := 7;
  NEW(zero);
  NEW(holder);
  head := NIL;
  FOR i := 1 TO 1000 DO
    NEW(temporary); temporary.value := i; temporary.next := head; head := temporary
  END;
  (* Hand the list to a pointer field of an array field of a record field of a
     heap object, and drop every pointer variable that referred to it. From
     here to the traversal below, the whole chain is retained only through the
     fields of scanned heap objects. *)
  holder.nested.links[0] := head; holder.nested.links[1] := NIL;
  head := NIL; temporary := NIL;
  FOR i := 1 TO 200000 DO
    NEW(temporary); temporary.value := i; temporary.next := NIL
  END;
  temporary := NIL;
  NEW(holder.selected); holder.selected.value := 11; holder.selected.next := NIL;
  NEW(holder.slots[Index()]); holder.slots[1].value := 12; holder.slots[1].next := NIL;
  count := 0; sum := 0;
  head := holder.nested.links[0];
  WHILE head # NIL DO INC(count); sum := sum + head.value; head := head.next END;
  Out.Int(atomic.value, 0); Out.Int(count, 0); Out.Int(sum, 0);
  Out.Int(holder.selected.value, 0); Out.Int(holder.slots[1].value, 0);
  Out.Int(indexCalls, 0); Out.Ln
END PointerGc.
