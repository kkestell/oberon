MODULE PointerForwardBad;
IMPORT Out;
TYPE
  MissingPointer = POINTER TO Missing;
  ArrayPointer = POINTER TO LaterArray;
  PointerPointer = POINTER TO LaterPointer;
  Qualified = POINTER TO Out.Missing;
  Plain = RECORD next: LaterRecord END;
  LaterArray = ARRAY 2 OF INTEGER;
  LaterPointer = POINTER TO Final;
  Final = RECORD n: INTEGER END;
VAR nestedOnly: POINTER TO ProcedureRecord;
PROCEDURE P;
TYPE ProcedureRecord = RECORD n: INTEGER END;
BEGIN
END P;
END PointerForwardBad.
