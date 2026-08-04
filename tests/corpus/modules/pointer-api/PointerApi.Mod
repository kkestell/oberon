MODULE PointerApi;
IMPORT PointerTypes, Out;

TYPE Other = POINTER TO PointerTypes.Record;

VAR p: PointerTypes.Pointer;
  q: Other;
  hidden: PointerTypes.HiddenPointer;

BEGIN
  p := PointerTypes.root;
  q := p;
  IF q = p THEN Out.Int(q.value, 0) END;
  p := PointerTypes.Identity(q);
  PointerTypes.Clear(p);
  IF p = PointerTypes.None THEN Out.Int(1, 0) END;
  NEW(hidden); hidden.public := 9; Out.Int(hidden.public, 0);
  Out.Ln
END PointerApi.
