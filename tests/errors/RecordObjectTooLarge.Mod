MODULE RecordObjectTooLarge;
  TYPE Block = ARRAY 134217728 OF INTEGER;
       TooLarge = RECORD block: Block; extra: BYTE END;
END RecordObjectTooLarge.
