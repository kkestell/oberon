MODULE RecordPrivate;
  IMPORT RecordPrivateSource;
  VAR own: RecordPrivateSource.Public; n: INTEGER;
BEGIN
  n := own.hidden;
  n := own.nested.hidden;
  n := RecordPrivateSource.public.hidden;
  n := RecordPrivateSource.private.hidden
END RecordPrivate.
