MODULE CaseLabelExpression;
  (* `label = integer | string | qualident`, so a constant expression is not
     a label. oberonc accepts this; that is an extension to the normative
     grammar and this compiler does not adopt it. *)
  CONST Max = 10;
  VAR k: INTEGER;
BEGIN
  CASE k OF Max - 1: END
END CaseLabelExpression.
