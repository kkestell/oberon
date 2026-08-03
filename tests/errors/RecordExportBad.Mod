MODULE RecordExportBad;
  PROCEDURE P;
    TYPE Local = RECORD public*: INTEGER END;
    VAR value: Local;
  BEGIN value.public := 1
  END P;
END RecordExportBad.
