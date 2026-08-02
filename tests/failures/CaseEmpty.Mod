MODULE CaseEmpty;
  (* `case = [CaseLabelList ":" StatementSequence]`, so a case statement with
     no labelled alternative parses. It can never match. *)
  VAR k: INTEGER;
BEGIN
  k := 1;
  CASE k OF END
END CaseEmpty.
