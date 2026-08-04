MODULE PointerPrivateSource;
TYPE P* = POINTER TO R; R = RECORD public*: INTEGER; private: INTEGER END;
VAR p*: P;
END PointerPrivateSource.
