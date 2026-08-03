MODULE CharByteConstCall;
  IMPORT Support;
  CONST
    A = Support.TakeChar("A");
    B = Support.TakeByte(255);
    C = Support.ReturnByte() + 1;
END CharByteConstCall.
