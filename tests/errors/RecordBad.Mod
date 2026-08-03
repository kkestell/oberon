MODULE RecordBad;
  TYPE
    R = RECORD n: INTEGER END;
    Other = RECORD n: INTEGER END;
    Duplicate = RECORD x: INTEGER; x: CHAR END;
  VAR r: R; other: Other; n: INTEGER; b: BOOLEAN;
      inline: RECORD n: INTEGER END;
      otherInline: RECORD n: INTEGER END;

  PROCEDURE Result(): R;
  END Result;

BEGIN
  b := r = r;
  b := r # r;
  b := r < r;
  n := r + r;
  IF r THEN n := 1 END;
  n := r.missing;
  n := n.field;
  r := other;
  inline := otherInline
END RecordBad.
