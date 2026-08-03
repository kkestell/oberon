MODULE RecordConstRelation;
  TYPE R = RECORD n: INTEGER END;
  VAR r: R;

  PROCEDURE P;
    CONST Equal = r = r;
  END P;
END RecordConstRelation.
