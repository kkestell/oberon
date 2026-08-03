MODULE ApiSupport;
  (* The exported side of a module: constants, variables, and both kinds of
     procedure. Everything unmarked is private and must stay invisible to a
     client. *)
  CONST
    Limit* = 10;
    Ready* = TRUE;
    Bias = 100;
  VAR
    count*, scratch: INTEGER;
    open*: BOOLEAN;

  PROCEDURE Bump*(by: INTEGER);
  BEGIN
    count := count + by
  END Bump;

  PROCEDURE Twice*(n: INTEGER): INTEGER;
  BEGIN
  RETURN n * 2
  END Twice;

  PROCEDURE Split*(n: INTEGER; VAR half, rest: INTEGER);
  BEGIN
    half := n DIV 2;
    rest := n MOD 2
  END Split;

  PROCEDURE Biased(n: INTEGER): INTEGER;
  BEGIN
  RETURN n + Bias
  END Biased;

BEGIN
  count := 1;
  scratch := Biased(0);
  open := TRUE
END ApiSupport.
