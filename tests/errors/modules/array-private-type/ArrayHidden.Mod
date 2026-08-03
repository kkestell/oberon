MODULE ArrayHidden;
  TYPE Shown* = ARRAY 2 OF INTEGER;
       Hidden = ARRAY 2 OF INTEGER;
  VAR ours: Hidden;
BEGIN
  ours[0] := 1
END ArrayHidden.
