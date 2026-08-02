MODULE CaseNegativeLabel;
  (* `integer` carries no sign, so a negative label has to be written as a
     named constant. Case.Mod shows the form that works. *)
  VAR k: INTEGER;
BEGIN
  CASE k OF -1: END
END CaseNegativeLabel.
