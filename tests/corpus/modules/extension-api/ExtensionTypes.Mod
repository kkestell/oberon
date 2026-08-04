MODULE ExtensionTypes;
  TYPE Base* = RECORD public*: INTEGER; private: INTEGER END;
       Child* = RECORD (Base) extra*: INTEGER END;
       PBase* = POINTER TO Base;
       PChild* = POINTER TO Child;

  PROCEDURE Change*(VAR value: Base; n: INTEGER);
  BEGIN value.public := n; value.private := n + 1 END Change;

  PROCEDURE IsChild*(VAR value: Base): BOOLEAN;
  RETURN value IS Child
  END IsChild;
END ExtensionTypes.
