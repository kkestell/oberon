MODULE RecordObjectTooLarge;
  TYPE Block = ARRAY 268435456 OF INTEGER;
       TooLarge = RECORD block: Block; extra: BYTE END;
END RecordObjectTooLarge.
