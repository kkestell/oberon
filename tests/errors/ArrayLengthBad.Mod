MODULE ArrayLengthBad;
  VAR
    n: INTEGER;
    negative: ARRAY -1 OF INTEGER;
    dynamic: ARRAY n OF INTEGER;
    wrongType: ARRAY TRUE OF INTEGER;
    invalid: ARRAY 1 DIV 0 OF INTEGER;
END ArrayLengthBad.
