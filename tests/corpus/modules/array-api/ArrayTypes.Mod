MODULE ArrayTypes;
  (* The exported side of an array interface: a named array type a client can
     declare its own variables with, and variables of a private type and of an
     inline type, which a client can read and index but cannot name. *)
  TYPE
    Row* = ARRAY 3 OF INTEGER;
    Hidden = ARRAY 2 OF INTEGER;
  VAR
    shared*: Row;
    secret*: Hidden;
    inline*: ARRAY 4 OF INTEGER;
    i: INTEGER;
BEGIN
  FOR i := 0 TO LEN(shared) - 1 DO shared[i] := i + 1 END;
  FOR i := 0 TO LEN(secret) - 1 DO secret[i] := i + 10 END;
  FOR i := 0 TO LEN(inline) - 1 DO inline[i] := i + 100 END
END ArrayTypes.
