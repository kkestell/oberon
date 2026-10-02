MODULE BuiltinBad;
  (* A shift count is diagnosed whenever lowering already knows it, which
     covers integer literals and named constants alike: both reach the IR as
     an immediate. Only other count expressions get the runtime check. *)
  CONST C = 1; Wide = 64; Folded = ABS(-9223372036854775808);
  VAR n: INTEGER; b: BOOLEAN;
BEGIN
  n := ABS(b);
  b := ODD(b);
  n := LSL(1, -1);
  n := ASR(1, 64);
  n := ROR(1, Wide);
  ASSERT(n);
  INC(C);
  INC(n, 1, 2);
  n := INC(n);
  ABS(n)
END BuiltinBad.
