MODULE Program;

IMPORT OberonRuntime;

VAR count*: INTEGER;

PROCEDURE Arg*(n: INTEGER; VAR arg: ARRAY OF CHAR; VAR res: INTEGER);
BEGIN
  ASSERT((0 <= n) & (n < count));
  res := OberonRuntime.ProgramArg(n, arg)
END Arg;

(* 0 through 255 is the range a POSIX parent can observe. *)
PROCEDURE Exit*(status: INTEGER);
BEGIN
  ASSERT((0 <= status) & (status <= 255));
  OberonRuntime.ProgramExit(status)
END Exit;

BEGIN
  count := OberonRuntime.ProgramArgCount()
END Program.
