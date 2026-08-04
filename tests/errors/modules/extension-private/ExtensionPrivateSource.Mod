MODULE ExtensionPrivateSource;
  TYPE Hidden = RECORD secret: INTEGER END;
       Public* = RECORD public*: INTEGER; private: INTEGER END;
       Child* = RECORD (Public) extra*: INTEGER END;
END ExtensionPrivateSource.
