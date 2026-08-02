MODULE Params;
IMPORT Out;
CONST
  NegSeven = -7;
  Two = 2;
  FoldDiv = NegSeven DIV Two;
  FoldMod = NegSeven MOD Two;
  Truth = TRUE;
VAR moduleInt, ticks: INTEGER; moduleBool: BOOLEAN;

PROCEDURE Flip(value: BOOLEAN): BOOLEAN;
BEGIN
  value := ~value
RETURN value
END Flip;

PROCEDURE Update(value: INTEGER; flag: BOOLEAN;
                 VAR outInt: INTEGER; VAR outBool: BOOLEAN): INTEGER;
CONST One = 1;
VAR localInt: INTEGER; localBool: BOOLEAN;
BEGIN
  value := value + One;
  flag := flag & Truth;
  localInt := value;
  localBool := Flip(flag);
  outInt := localInt;
  outBool := localBool
RETURN value
END Update;

PROCEDURE Tick;
BEGIN
  ticks := ticks + 1
END Tick;

PROCEDURE Run;
VAR x, y, result: INTEGER; flag: BOOLEAN;
BEGIN
  moduleInt := 10; moduleBool := TRUE;
  result := Update(moduleInt, moduleBool, x, flag);
  Out.Int(moduleInt, 0);
  IF moduleBool THEN Out.Int(1, 2) ELSE Out.Int(0, 2) END;
  Out.Int(result, 3); Out.Int(x, 3);
  IF flag THEN Out.Int(1, 2) ELSE Out.Int(0, 2) END;
  Out.Ln;

  x := NegSeven; y := Two;
  Out.Int(FoldDiv, 0); Out.Int(x DIV y, 3);
  Out.Int(FoldMod, 2); Out.Int(x MOD y, 2); Out.Ln;

  Tick; Tick(); Out.Int(ticks, 0); Out.Ln
END Run;

BEGIN
  ticks := 0; Run
END Params.
