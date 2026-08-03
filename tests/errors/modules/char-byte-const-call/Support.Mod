MODULE Support;
  PROCEDURE TakeChar*(ch: CHAR): INTEGER;
  BEGIN
  RETURN ORD(ch)
  END TakeChar;

  PROCEDURE TakeByte*(b: BYTE): INTEGER;
  BEGIN
  RETURN b
  END TakeByte;

  PROCEDURE ReturnByte*(): BYTE;
  BEGIN
  RETURN 1
  END ReturnByte;
END Support.
