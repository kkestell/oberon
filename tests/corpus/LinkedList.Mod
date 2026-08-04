MODULE LinkedList;
IMPORT Out;

TYPE
  Link = POINTER TO Node;
  Node = RECORD value: INTEGER; next: Link END;

VAR head: Link;

PROCEDURE Push(VAR list: Link; value: INTEGER);
VAR node: Link;
BEGIN
  NEW(node);
  node.value := value;
  node.next := list;
  list := node
END Push;

PROCEDURE Remove(VAR list: Link);
BEGIN
  IF list # NIL THEN list := list.next END
END Remove;

BEGIN
  head := NIL;
  Push(head, 1); Push(head, 2); Push(head, 3);
  WHILE (head # NIL) & (head^.value > 0) DO
    Out.Int(head.value, 0);
    Remove(head)
  END;
  IF head = NIL THEN Out.Int(0, 0) END;
  Out.Ln
END LinkedList.
