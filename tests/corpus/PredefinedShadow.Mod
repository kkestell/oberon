MODULE PredefinedShadow;
  (* Report 4 forbids two objects sharing an identifier within one scope, and
     the predefined identifiers are not declared in any module's scope. They
     live in a universe scope that the module scope is opened inside, so a
     module may declare its own ABS and shadow the predefined one. While the
     predefined names sat in the module scope instead, every declaration
     below was an "already declared" error.

     The declarations are at module level on purpose. A local one would prove
     nothing: locals already shadow module-level names. *)
  IMPORT Out;
  CONST ODD = 7; INCL = 9;
  VAR ABS: INTEGER;
      BOOLEAN: INTEGER;
      SET: INTEGER;

  PROCEDURE ASSERT(n: INTEGER): INTEGER;
  BEGIN
  RETURN n * 2
  END ASSERT;

  PROCEDURE EXCL(n: INTEGER): INTEGER;
  BEGIN
  RETURN n + 1
  END EXCL;

BEGIN
  ABS := 3; BOOLEAN := 4;
  Out.Int(ABS, 4); Out.Int(BOOLEAN, 4); Out.Int(ODD, 4); Out.Int(ASSERT(5), 4);
  Out.Ln;
  SET := 5;
  Out.Int(SET, 4); Out.Int(INCL, 4); Out.Int(EXCL(6), 4); Out.Ln
END PredefinedShadow.
