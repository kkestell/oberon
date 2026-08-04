MODULE DynamicTypes;
  IMPORT Out;
  TYPE Root = RECORD a: INTEGER END;
       Child = RECORD (Root) b: INTEGER END;
       Grand = RECORD (Child) c: INTEGER END;
       Sibling = RECORD (Root) s: INTEGER END;
       PRoot = POINTER TO Root;
       PChild = POINTER TO Child;
       PGrand = POINTER TO Grand;
       PSibling = POINTER TO Sibling;
  VAR calls: INTEGER; p: PRoot; pc: PChild; pg: PGrand; ps: PSibling;
      r: Root; c: Child; g: Grand;

  PROCEDURE Result(): PRoot;
  BEGIN INC(calls); RETURN p END Result;

  PROCEDURE RecordCase(VAR x: Root);
  BEGIN
    CASE x OF
      Child: Out.Int(x.b, 0)
    | Grand: Out.Int(99, 0)
    | Root: Out.Int(x.a, 0)
    END;
    Out.Ln
  END RecordCase;

  PROCEDURE Forward(VAR x: Root);
  BEGIN
    Out.Int(ORD(x IS Grand), 0);
    RecordCase(x)
  END Forward;

  PROCEDURE ThroughPointer(VAR x: Root);
  BEGIN Out.Int(ORD(x IS Child), 0); Out.Int(x(Child).b, 0); Out.Ln
  END ThroughPointer;

  PROCEDURE RecordNoMatch(VAR x: Root);
  BEGIN
    CASE x OF Child: Out.Int(99, 0) END;
    CASE x OF END;
    Out.Int(50, 0); Out.Ln
  END RecordNoMatch;

  PROCEDURE Nested(VAR x: Root);
  BEGIN
    CASE x OF Child:
      CASE x OF Grand: Out.Int(x.c, 0) | Child: Out.Int(x.b, 0) END
    END;
    Out.Int(x.a, 0); Out.Ln
  END Nested;

BEGIN
  c.a := 1; c.b := 2;
  g.a := 3; g.b := 4; g.c := 5;
  Forward(g);
  RecordCase(c);
  RecordCase(r);
  RecordNoMatch(r);
  Nested(g);

  NEW(pc); pc.a := 6; pc.b := 7; p := pc;
  Out.Int(ORD(Result() IS PChild), 0); Out.Int(calls, 0); Out.Ln;
  ThroughPointer(p^);
  NEW(ps); p := ps; Out.Int(ORD(p IS PChild), 0); Out.Ln; p := pc;

  CASE p OF
    PRoot: Out.Int(10, 0)
  | PChild: Out.Int(20, 0)
  END;
  Out.Ln;
  CASE p OF | PChild: Out.Int(p.b, 0) END; Out.Ln;

  p := NIL;
  CASE p OF PChild: Out.Int(30, 0) END;
  CASE p OF END;
  Out.Int(40, 0); Out.Ln;

  NEW(pc); p := pc;
  NEW(p(PChild)); p(PChild).b := 8;
  CASE p OF PChild: NEW(p); p.b := 9 END;
  Out.Int(p(PChild).b, 0); Out.Ln
END DynamicTypes.
