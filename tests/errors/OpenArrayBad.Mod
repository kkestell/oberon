MODULE OpenArrayBad;
TYPE A = ARRAY 2 OF INTEGER;
     B = ARRAY 2 OF INTEGER;
     R = RECORD n: INTEGER END;
     S = RECORD n: INTEGER END;
VAR n: INTEGER;
    ints: ARRAY 3 OF INTEGER;
    reals: ARRAY 3 OF REAL;
    grid: ARRAY 2, 3 OF INTEGER;
    aa: ARRAY 1 OF A;
    bb: ARRAY 1 OF B;
    rr: ARRAY 1 OF R;
    ss: ARRAY 1 OF S;

PROCEDURE One(a: ARRAY OF INTEGER); END One;
PROCEDURE Two(a: ARRAY OF ARRAY OF INTEGER); END Two;
PROCEDURE NamedArray(a: ARRAY OF A); END NamedArray;
PROCEDURE NamedRecord(a: ARRAY OF R); END NamedRecord;
PROCEDURE V(VAR a: ARRAY OF CHAR); END V;
BEGIN
  One(n);
  One(reals);
  One(grid);
  Two(ints);
  NamedArray(bb);
  NamedRecord(ss);
  One("bad");
  Two("bad");
  V("bad");
  One;
  One(ints, ints)
END OpenArrayBad.
