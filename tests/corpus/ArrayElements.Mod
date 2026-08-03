MODULE ArrayElements;
  (* An array of each scalar type, in module storage and in an activation
     record. A selected element is an ordinary writable variable: it is
     assigned, passed to a scalar VAR parameter, and changed by every
     predefined operation that changes its argument. REAL elements are
     observed through FLOOR and SET elements through ORD, because Out has
     only Int. *)
  IMPORT Out;
  VAR ints: ARRAY 3 OF INTEGER;
      flags: ARRAY 3 OF BOOLEAN;
      sets: ARRAY 3 OF SET;
      reals: ARRAY 3 OF REAL;

  PROCEDURE Double(VAR n: INTEGER);
  BEGIN n := n * 2
  END Double;

  PROCEDURE Show(b: BOOLEAN): INTEGER;
  BEGIN
  RETURN ORD(b)
  END Show;

  PROCEDURE Locals;
    (* The same operations against an array that lives in the frame. *)
    VAR local: ARRAY 4 OF INTEGER;
        temperatures: ARRAY 2 OF REAL;
        k: INTEGER;
  BEGIN
    FOR k := 0 TO LEN(local) - 1 DO local[k] := k + 1 END;
    Double(local[2]);
    INC(local[0], 5); DEC(local[3]);
    FOR k := 0 TO LEN(local) - 1 DO Out.Int(local[k], 4) END;
    Out.Ln;
    temperatures[0] := 1.5; temperatures[1] := temperatures[0] * 3.0;
    PACK(temperatures[1], 2);
    Out.Int(FLOOR(temperatures[0]), 4); Out.Int(FLOOR(temperatures[1]), 4);
    Out.Ln
  END Locals;

BEGIN
  ints[0] := 7; ints[1] := 8; ints[2] := 9;
  Double(ints[1]); INC(ints[0]); DEC(ints[2], 3);
  Out.Int(ints[0], 4); Out.Int(ints[1], 4); Out.Int(ints[2], 4); Out.Ln;

  flags[0] := TRUE; flags[1] := FALSE; flags[2] := flags[0] & ~flags[1];
  Out.Int(Show(flags[0]), 4); Out.Int(Show(flags[1]), 4);
  Out.Int(Show(flags[2]), 4); Out.Ln;

  sets[0] := {1, 3}; INCL(sets[0], 5); EXCL(sets[0], 1);
  sets[1] := sets[0] * {3}; sets[2] := sets[0] + sets[1];
  Out.Int(ORD(sets[0]), 4); Out.Int(ORD(sets[1]), 4); Out.Int(ORD(sets[2]), 4);
  Out.Ln;

  (* 12.0 is 1.5 * 2^3, so UNPK stores the fraction in one element and the
     exponent in another, and both of them are selected. *)
  reals[0] := 12.0;
  UNPK(reals[0], ints[2]);
  Out.Int(FLOOR(reals[0] * 10.0), 4); Out.Int(ints[2], 4); Out.Ln;
  Locals
END ArrayElements.
