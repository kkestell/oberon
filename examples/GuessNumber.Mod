(*  A binary search for the number you are thinking of. Answer h, l, or c on
    standard input.

    Source: lboasso/oberonc (MIT), examples/GuessNumber.Mod.

    Changed: Reads the name with In.Name rather than In.String, which in this
    library reads a quoted string. CharLn answers "c" at end of input instead
    of spinning.
*)

MODULE GuessNumber;
  IMPORT In, Out;
  VAR
    name: ARRAY 20 OF CHAR;
    number, left, right, old: INTEGER;
    choice: CHAR;

(* Read one character and drop the rest of its line. On end of input,
   answer "c" so the loop below stops instead of spinning. *)
PROCEDURE CharLn(VAR ch: CHAR);
VAR discard: CHAR;
BEGIN
  In.Char(ch);
  IF ~In.Done THEN ch := "c"
  ELSE
    REPEAT
      In.Char(discard)
    UNTIL ~In.Done OR (discard = 0AX)
  END
END CharLn;

BEGIN
  Out.String("What's your name? ");
  In.Name(name);
  Out.String("Hi "); Out.String(name); Out.Char("!"); Out.Ln;
  Out.String("Please think of a number from 0 to 50 and I'll guess it.");
  Out.Ln;
  left := 0;
  right := 50;
  number := 27;
  REPEAT
    REPEAT
      Out.String("Is "); Out.Int(number, 3);
      Out.String(" the correct number? [(h)igher (l)ower (c)orrect] ");
      CharLn(choice)
    UNTIL (choice = "h") OR (choice = "l") OR (choice = "c");
    IF choice = "h" THEN left := number + 1
    ELSIF choice = "l" THEN right := number - 1
    END ;
    old := number;
    number := left + (right - left) DIV 2;
    IF (choice # "c") & (number = old) THEN
       Out.Ln; Out.String("You lied :)");
       choice := "c"
    END
  UNTIL choice = "c";
  Out.Ln
END GuessNumber.
