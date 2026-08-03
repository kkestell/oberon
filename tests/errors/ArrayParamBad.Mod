MODULE ArrayParamBad;
  (* Slice 12 gives structured parameters their calling convention along with
     records. Until then both forms stop here rather than reaching the scalar
     ABI. *)
  TYPE Row = ARRAY 3 OF INTEGER;

  PROCEDURE ByValue(a: Row);
  END ByValue;

  PROCEDURE ByReference(VAR a: Row);
  END ByReference;

END ArrayParamBad.
