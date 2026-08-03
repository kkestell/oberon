MODULE CharTypes;
  (* The exported side of a character interface: a named character array
     type, a CHAR constant, a string constant, and CHAR and BYTE variables
     the body initializes. *)
  CONST
    Mark* = CHR(33);
    Greeting* = "hello";
  TYPE Name* = ARRAY 8 OF CHAR;
  VAR
    label*: Name;
    initial*: CHAR;
    flags*: BYTE;
BEGIN
  label := "chars"; initial := "C"; flags := 200
END CharTypes.
