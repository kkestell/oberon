MODULE RecordApi;
  IMPORT Out, RecordTypes;
  VAR mine: RecordTypes.Public;
BEGIN
  mine := RecordTypes.shared;
  Out.Int(RecordTypes.Read(mine), 0); Out.Int(RecordTypes.Read(RecordTypes.shared), 4);
  Out.Int(mine.shown, 4); Out.Int(mine.row[1], 4); Out.Int(mine.nested.shown, 4); Out.Ln;
  RecordTypes.Change(mine, 20);
  Out.Int(RecordTypes.Read(mine), 0); Out.Int(RecordTypes.shared.shown, 4); Out.Ln;
  Out.Int(mine.shown, 0); Out.Int(RecordTypes.shared.shown, 4);
  Out.Int(RecordTypes.private.value, 4); Out.Ln
END RecordApi.
