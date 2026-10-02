MODULE LibraryOut;

IMPORT Out;

TYPE Writer = PROCEDURE(ch: CHAR);

VAR write: Writer; full: ARRAY 4 OF CHAR; inf, nan: REAL;

BEGIN
  Out.Open;
  write := Out.Char;
  write("A"); Out.Ln;
  Out.String("text"); Out.Ln;
  full[0] := "b"; full[1] := "o"; full[2] := "u"; full[3] := "n";
  Out.String(full); Out.Ln;
  Out.Int(7, 0); Out.Char("|"); Out.Int(7, -3); Out.Char("|");
  Out.Int(-9223372036854775807 - 1, 22); Out.Char("|"); Out.Int(9223372036854775807, 0); Out.Ln;
  Out.Hex(-1); Out.Hex(12345678H); Out.Ln;
  Out.Real(1.5, 0); Out.Ln;
  Out.Real(-0.0, 23); Out.Ln;
  inf := 1.0 / 0.0; nan := 0.0 / 0.0;
  (* A NaN's sign bit is whatever the target's invalid operation produced, and
     no Oberon operation can read or set it. ABS clears it, so the spelling
     Out.Real writes for a NaN is the same on every target. *)
  Out.Real(inf, 0); Out.Char("|"); Out.Real(-inf, 0); Out.Char("|");
  Out.Real(ABS(nan), 0); Out.Ln
END LibraryOut.
