MODULE RecordExtensionBad;
  TYPE Base = RECORD n: INTEGER END;
       Derived = RECORD (Base) extra: INTEGER END;
END RecordExtensionBad.
