MODULE SetProcedures;
  (* SET in every storage and procedure role the compiler has: a module
     variable, a local, a value parameter, a VAR parameter, an argument to a
     proper procedure, and a function result. *)
  IMPORT Out;
  VAR global: SET;

  PROCEDURE Show(s: SET);
  BEGIN
    Out.Int(ORD(s), 14)
  END Show;

  (* Report 10.1: a value parameter is a writable local copy, so INCL and
     EXCL here must leave the caller's variable alone. *)
  PROCEDURE Consume(s: SET): INTEGER;
  BEGIN
    INCL(s, 10);
    EXCL(s, 0)
  RETURN ORD(s)
  END Consume;

  PROCEDURE Update(VAR s: SET; add, drop: INTEGER);
  BEGIN
    INCL(s, add);
    EXCL(s, drop)
  END Update;

  PROCEDURE Between(low, high: INTEGER): SET;
    VAR local: SET;
  BEGIN
    local := {low .. high}
  RETURN local
  END Between;

BEGIN
  global := {0, 1};
  Show(global); Out.Ln;
  Out.Int(Consume(global), 14); Show(global); Out.Ln;
  Update(global, 4, 0);
  Show(global); Out.Ln;
  Show(Between(2, 4)); Show(Between(4, 2)); Out.Ln
END SetProcedures.
