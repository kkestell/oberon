MODULE TypeCaseBad;
  TYPE Root = RECORD value: INTEGER END;
       Child = RECORD (Root) child: INTEGER END;
       Other = RECORD other: INTEGER END;
       PRoot = POINTER TO Root;
       PChild = POINTER TO Child;
       POther = POINTER TO Other;
       Holder = RECORD pointer: PRoot END;
  VAR root: Root; pointer: PRoot; holder: Holder; n: INTEGER; flag: BOOLEAN; target: PChild;

  PROCEDURE Value(value: Root);
  BEGIN CASE value OF Child: END END Value;

  PROCEDURE RecordLabels(VAR value: Root);
  BEGIN
    CASE value OF PChild: | target: | Other: END
  END RecordLabels;

  PROCEDURE Result(): PRoot;
  RETURN pointer
  END Result;
BEGIN
  CASE root OF Child: END;
  CASE holder.pointer OF PChild: END;
  CASE n OF PChild: END;
  CASE flag OF 0: END;
  CASE Result() OF PChild: END;
  CASE pointer OF target: END;
  CASE pointer OF Child: END;
  CASE pointer OF POther: END;
  CASE pointer OF PChild, PRoot: END;
  CASE pointer OF PChild .. PRoot: END;
  CASE pointer OF PChild: | PChild: END
END TypeCaseBad.
