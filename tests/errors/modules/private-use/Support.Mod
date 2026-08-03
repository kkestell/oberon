MODULE Support;
  CONST Public* = 1; Secret = 2;
  VAR shown*, hidden: INTEGER;

  PROCEDURE Shown*;
  END Shown;

  PROCEDURE Concealed;
  END Concealed;

BEGIN
  shown := Public; hidden := Secret
END Support.
