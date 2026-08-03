MODULE ArrayLenConstIndexBad;
  VAR a: ARRAY 2 OF ARRAY 3 OF INTEGER;
      bad: BOOLEAN;

  PROCEDURE P;
    CONST Mixed = LEN(a[1 + TRUE]);
          Wrong = LEN(a[bad]);
  END P;

END ArrayLenConstIndexBad.
