MODULE HexLiteralRange;
  (* A hex literal is a 64-bit pattern, so a seventeenth digit cannot fit.
     The lexer rejects it, so it cannot share a module with a diagnostic
     that semantic analysis reports. *)
  VAR n: INTEGER;
BEGIN
  n := 0FFFFFFFFFFFFFFFFH;
  n := 10000000000000000H
END HexLiteralRange.
