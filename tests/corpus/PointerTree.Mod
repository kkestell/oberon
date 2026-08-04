MODULE PointerTree;
IMPORT Out;

TYPE
  Tree = POINTER TO Node;
  Node = RECORD key: INTEGER; left, right: Tree END;

VAR root: Tree;

PROCEDURE Insert(VAR tree: Tree; key: INTEGER);
BEGIN
  IF tree = NIL THEN
    NEW(tree); tree.key := key; tree.left := NIL; tree.right := NIL
  ELSIF key < tree.key THEN Insert(tree.left, key)
  ELSIF key > tree.key THEN Insert(tree.right, key)
  END
END Insert;

PROCEDURE Find(tree: Tree; key: INTEGER): Tree;
VAR found: Tree;
BEGIN
  IF tree = NIL THEN found := NIL
  ELSIF key = tree.key THEN found := tree
  ELSIF key < tree.key THEN found := Find(tree.left, key)
  ELSE found := Find(tree.right, key)
  END
RETURN found
END Find;

PROCEDURE Print(tree: Tree);
BEGIN
  IF tree # NIL THEN Print(tree.left); Out.Int(tree.key, 0); Print(tree.right) END
END Print;

BEGIN
  root := NIL;
  Insert(root, 4); Insert(root, 2); Insert(root, 6); Insert(root, 1); Insert(root, 3);
  Print(root);
  IF Find(root, 3) # NIL THEN Out.Int(1, 0) END;
  IF Find(root, 9) = NIL THEN Out.Int(0, 0) END;
  Out.Ln
END PointerTree.
