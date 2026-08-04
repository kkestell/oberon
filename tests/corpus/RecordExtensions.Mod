MODULE RecordExtensions;
  IMPORT Out;
  TYPE Root = RECORD a: INTEGER END;
       Child = RECORD (Root) b: INTEGER END;
       Alias = Child;
       Grand = RECORD (Child) c: INTEGER END;
       Sibling = RECORD (Root) s: INTEGER END;
       PRoot = POINTER TO Root;
       PChild = POINTER TO Child;
       PGrand = POINTER TO Grand;
       Empty = RECORD END;
       EmptyChild = RECORD (Empty) END;
       PEmpty = POINTER TO Empty;
       PEmptyChild = POINTER TO EmptyChild;
  VAR r: Root; c: Child; alias: Alias; g: Grand;
      p: PRoot; pc: PChild; pg: PGrand;
      e: PEmpty; ec: PEmptyChild;

  PROCEDURE Value(r: Root): INTEGER;
  BEGIN RETURN r.a END Value;

  PROCEDURE CopyBase(VAR destination: Root; source: Root);
  BEGIN destination := source END CopyBase;

  PROCEDURE Local;
    TYPE LocalChild = RECORD (Root) n: INTEGER END;
    VAR x: LocalChild;
  BEGIN x.a := 7; x.n := 8; Out.Int(x.a + x.n, 0); Out.Ln
  END Local;

  PROCEDURE ConstantNil;
    CONST NilValue = NIL; IsPointer = NilValue IS PRoot;
  BEGIN Out.Int(ORD(IsPointer), 0); Out.Ln END ConstantNil;

BEGIN
  c.a := 1; c.b := 2;
  alias := c;
  g.a := 3; g.b := 4; g.c := 5;
  r := c; Out.Int(r.a, 0); Out.Ln;
  c := g; Out.Int(c.a + c.b, 0); Out.Ln;
  c.b := 20; CopyBase(c, g); Out.Int(c.a + c.b, 0); Out.Ln;
  Out.Int(Value(g), 0); Out.Ln;

  NEW(pc); pc.a := 10; pc.b := 20; p := pc;
  Out.Int(ORD(p = pc), 0); Out.Int(ORD(pc = p), 0); Out.Ln;
  Out.Int(ORD(p IS PChild), 0); Out.Int(ORD(p IS PGrand), 0); Out.Ln;
  Out.Int(p(PRoot)(PChild).b, 0); Out.Ln;
  pg := NIL; Out.Int(ORD(pg IS PGrand), 0); Out.Ln; ConstantNil;

  NEW(ec); e := ec; NEW(ec);
  Out.Int(ORD(e # ec), 0); Out.Int(ORD(e IS PEmptyChild), 0); Out.Ln;
  Local
END RecordExtensions.
