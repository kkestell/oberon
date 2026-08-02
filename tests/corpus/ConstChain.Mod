MODULE ConstChain;
  IMPORT Out;
  CONST A = 2; B = A * 3; C = B + A;
  VAR x: INTEGER;
BEGIN
  x := C;
  Out.Int(x, 0); Out.Ln
END ConstChain.
