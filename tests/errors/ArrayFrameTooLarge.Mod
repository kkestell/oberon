MODULE ArrayFrameTooLarge;
  (* Each local is half the limit, so the third one is where the activation
     record leaves what the target can address. *)
  TYPE Half = ARRAY 67108864 OF INTEGER;

  PROCEDURE Deep;
    VAR first, second, third: Half;
  BEGIN
  END Deep;

END ArrayFrameTooLarge.
