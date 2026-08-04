MODULE ExtensionApi;
  IMPORT Out, ExtensionTypes;
  TYPE Local = RECORD (ExtensionTypes.Base) private: INTEGER; local: INTEGER END;
       PLocal = POINTER TO Local;
  VAR child: ExtensionTypes.Child; local: Local;
      base: ExtensionTypes.PBase; p: ExtensionTypes.PChild; pl: PLocal;
BEGIN
  child.public := 1; child.extra := 2;
  ExtensionTypes.Change(child, 10);
  Out.Int(child.public, 0); Out.Int(child.extra, 0); Out.Int(ORD(ExtensionTypes.IsChild(child)), 0); Out.Ln;
  local.public := 3; local.private := 5; local.local := 4; ExtensionTypes.Change(local, 20);
  Out.Int(local.public, 0); Out.Int(local.private, 0); Out.Int(local.local, 0); Out.Int(ORD(ExtensionTypes.IsChild(local)), 0); Out.Ln;
  NEW(p); p.public := 5; p.extra := 6; base := p;
  Out.Int(ORD(base IS ExtensionTypes.PChild), 0);
  CASE base OF ExtensionTypes.PChild: Out.Int(base.extra, 0) END; Out.Ln;
  NEW(pl); pl.public := 7; pl.local := 8; base := pl;
  Out.Int(ORD(base IS PLocal), 0); Out.Int(base(PLocal).local, 0); Out.Ln
END ExtensionApi.
