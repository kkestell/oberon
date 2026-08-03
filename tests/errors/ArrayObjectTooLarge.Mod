MODULE ArrayObjectTooLarge;
  (* Every dimension is fine on its own and the product fits in 64-bit
     arithmetic, but the object is four gibibytes. *)
  TYPE Big = ARRAY 1024, 1024, 1024 OF INTEGER;
END ArrayObjectTooLarge.
