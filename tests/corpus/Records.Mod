MODULE Records;
  IMPORT Out;
  TYPE
    Empty = RECORD END;
    Inner = RECORD n: INTEGER; ch: CHAR END;
    Everything = RECORD
      i: INTEGER; r: REAL; b: BOOLEAN; s: SET; ch: CHAR; byte: BYTE;
      inner: Inner; row: ARRAY 3 OF INTEGER
    END;
    ZeroField = RECORD n: INTEGER; z: ARRAY 0 OF INTEGER END;
  VAR source, copy: Everything;
      empty, alsoEmpty: Empty;
      zero, alsoZero: ZeroField;

  PROCEDURE Local;
    VAR value: RECORD n: INTEGER; nested: Inner END;
  BEGIN
    value.n := 17; value.nested.n := 18;
    Out.Int(value.n, 0); Out.Int(value.nested.n, 4); Out.Ln
  END Local;

BEGIN
  source.i := 11; source.r := 2.5; source.b := TRUE; source.s := {1, 3};
  source.ch := "Q"; source.byte := 200; source.inner.n := 12;
  source.inner.ch := "R"; source.row[0] := 13; source.row[1] := 14;
  source.row[2] := 15;
  copy := source;
  source.i := 99; source.inner.n := 98; source.row[1] := 97;
  Out.Int(copy.i, 0); Out.Int(FLOOR(copy.r), 4); Out.Int(ORD(copy.b), 4);
  Out.Int(ORD(copy.s), 4); Out.Char(copy.ch); Out.Int(copy.byte, 4); Out.Ln;
  Out.Int(copy.inner.n, 0); Out.Char(copy.inner.ch);
  Out.Int(copy.row[0], 4); Out.Int(copy.row[1], 4); Out.Int(copy.row[2], 4); Out.Ln;
  empty := alsoEmpty;
  zero.n := 21; alsoZero := zero; zero.n := 22;
  Out.Int(alsoZero.n, 0); Out.Int(LEN(alsoZero.z), 4); Out.Ln;
  Local
END Records.
