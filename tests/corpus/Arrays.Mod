MODULE Arrays;
  (* A sieve of Eratosthenes over a named module-level BOOLEAN array. Every
     write and almost every read goes through an index the compiler cannot
     fold, so the checked-index lowering carries the whole program. LEN is the
     bound of every loop, which keeps the array's length in one place. *)
  IMPORT Out;
  CONST Limit = 50;
  TYPE Flags = ARRAY Limit OF BOOLEAN;
  VAR composite: Flags;
      i, j, count, checksum: INTEGER;
BEGIN
  FOR i := 0 TO LEN(composite) - 1 DO composite[i] := FALSE END;
  i := 2;
  WHILE i * i < LEN(composite) DO
    IF ~composite[i] THEN
      j := i * i;
      WHILE j < LEN(composite) DO composite[j] := TRUE; j := j + i END
    END;
    i := i + 1
  END;
  count := 0; checksum := 0;
  FOR i := 2 TO LEN(composite) - 1 DO
    IF ~composite[i] THEN
      count := count + 1; checksum := checksum + i;
      Out.Int(i, 4);
      IF count MOD 10 = 0 THEN Out.Ln END
    END
  END;
  Out.Ln;
  Out.Int(count, 0); Out.Int(checksum, 6); Out.Ln
END Arrays.
