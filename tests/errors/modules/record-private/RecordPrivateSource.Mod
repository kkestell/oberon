MODULE RecordPrivateSource;
  TYPE Public* = RECORD
         shown*: INTEGER; hidden: INTEGER;
         nested*: RECORD shown*: INTEGER; hidden: INTEGER END
       END;
       Private = RECORD shown*: INTEGER; hidden: INTEGER END;
  VAR public*: Public;
      private*: Private;
END RecordPrivateSource.
