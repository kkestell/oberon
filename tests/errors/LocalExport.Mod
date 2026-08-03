MODULE LocalExport;
  (* Report 4 allows an export mark only on a declaration in the module's
     scope. The parser accepts the syntax at every identdef; sema rejects
     these three. *)

  PROCEDURE Outer;
  CONST Local* = 1;
  VAR n*: INTEGER;

    PROCEDURE Inner*;
    END Inner;

  BEGIN
    n := Local
  END Outer;

END LocalExport.
