MODULE RealProcedures;
  (* REAL in every storage and procedure role the compiler has: a module
     variable, a local, a value parameter, a VAR parameter, an argument to a
     proper procedure, and a function result. *)
  IMPORT Out;
  VAR global: REAL;

  PROCEDURE Show(x: REAL);
  BEGIN
    Out.Int(FLOOR(x), 10)
  END Show;

  (* Report 10.1: a value parameter is a writable local copy, so doubling it
     here must leave the caller's variable alone. *)
  PROCEDURE Consume(x: REAL): INTEGER;
  BEGIN
    x := x * 2.
  RETURN FLOOR(x)
  END Consume;

  PROCEDURE Scale(VAR x: REAL; by: REAL);
  BEGIN
    x := x * by
  END Scale;

  PROCEDURE Sum(a, b: REAL): REAL;
    VAR local: REAL;
  BEGIN
    local := a + b
  RETURN local
  END Sum;

BEGIN
  global := 3.;
  Show(global); Out.Ln;
  Out.Int(Consume(global), 10); Show(global); Out.Ln;
  Scale(global, 4.);
  Show(global); Out.Ln;
  Show(Sum(1.5, 2.5)); Out.Ln
END RealProcedures.
