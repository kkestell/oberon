MODULE PointerSource;
TYPE P* = POINTER TO R; R* = RECORD n*: INTEGER END; Holder* = RECORD p*: P END;
VAR p*: P; holder*: Holder;
END PointerSource.
