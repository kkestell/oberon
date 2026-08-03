MODULE CharCase;
  (* Report 9.5: a CHAR selector with single labels, ranges, a CHAR constant
     label, and an ordinal literal label. Every alternative is reached. *)
  IMPORT Out;
  CONST Star = CHR(42);

  PROCEDURE Classify(c: CHAR): CHAR;
    VAR r: CHAR;
  BEGIN
    CASE c OF
      "a" .. "z": r := "L"
    | "A" .. "Z": r := "U"
    | 30X .. 39X: r := "D"
    | Star: r := "S"
    | 7EX: r := "T"
    | "?": r := "Q"
    END
  RETURN r
  END Classify;

BEGIN
  Out.Char(Classify("m"));
  Out.Char(Classify("Z"));
  Out.Char(Classify(39X));
  Out.Char(Classify("*"));
  Out.Char(Classify(7EX));
  Out.Char(Classify("?"));
  Out.Ln
END CharCase.
