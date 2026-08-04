MODULE RecordTypeGuardFail;
  TYPE Root = RECORD value: INTEGER END;
       Child = RECORD (Root) extra: INTEGER END;
  VAR root: Root;

  PROCEDURE Guard(VAR value: Root);
  BEGIN value(Child).extra := 1 END Guard;
BEGIN
  Guard(root)
END RecordTypeGuardFail.
