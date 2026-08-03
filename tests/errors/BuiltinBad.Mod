MODULE BuiltinBad;
  (* A shift count is diagnosed whenever lowering already knows it, which
     covers integer literals and named constants alike: both reach the IR as
     an immediate. Only other count expressions get the runtime check. *)
  CONST C = 1; Wide = 32; Folded = ABS(-2147483648);
  VAR n: INTEGER; b: BOOLEAN;
BEGIN
  n := ABS(b);
  b := ODD(b);
  n := LSL(1, -1);
  n := ASR(1, 32);
  n := ROR(1, Wide);
  ASSERT(n);
  INC(C);
  INC(n, 1, 2);
  n := INC(n);
  ABS(n)
END BuiltinBad.
