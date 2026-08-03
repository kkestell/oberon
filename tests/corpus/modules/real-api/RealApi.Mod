MODULE RealApi;
  (* The client folds the imported REAL constant, reads the imported variable
     the support module's body initialized, calls both exported procedure
     forms, and takes the REAL result of the function. Every result reaches
     the output through FLOOR. *)
  IMPORT Out, R := RealSupport;
  CONST Doubled = R.Ratio * 2.;
  VAR local: REAL;
BEGIN
  Out.Int(FLOOR(Doubled), 10); Out.Int(FLOOR(R.Ratio * 2.), 10); Out.Ln;
  Out.Int(FLOOR(R.scale), 10); Out.Ln;
  local := R.scale;
  R.Apply(R.Ratio, local);
  Out.Int(FLOOR(local), 10); Out.Ln;
  Out.Int(FLOOR(R.Sum(1.5, 2.5)), 10); Out.Ln
END RealApi.
