MODULE ProgramArgRange;

IMPORT Program;

VAR arg: ARRAY 8 OF CHAR; res: INTEGER;

BEGIN
  Program.Arg(0, arg, res)
END ProgramArgRange.
