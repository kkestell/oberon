MODULE Case;
  (* Report 9.5, INTEGER form. The loop drives one case statement through
     several of its alternatives in a single run. The alternatives are
     written out of order, one carries several labels, two ranges are
     adjacent but disjoint, one alternative has no labels at all, and the arm
     labelled 1 has an empty statement sequence: it matches, does nothing,
     and falls out of the case statement without trapping.

     Every negative label is written as a named constant. The grammar's
     `label` production is an integer, a string, or a qualident, so `-1` is
     not a label. *)
  IMPORT Out;
  CONST Low = -3; Neg = -2; Mid = -1; Far = 40;
  VAR k, calls: INTEGER;

  PROCEDURE Selector(v: INTEGER): INTEGER;
  BEGIN
    calls := calls + 1
  RETURN v
  END Selector;

BEGIN
  FOR k := -3 TO 6 DO
    Out.Int(k, 4);
    CASE k OF
      Far: Out.Int(999, 5)
    | 3..4: Out.Int(34, 5)
    | 5..6: Out.Int(56, 5)
    | Neg, 0: Out.Int(20, 5)
    |
    | 1:
    | Low, Mid: Out.Int(31, 5)
    | 2: Out.Int(2, 5)
    END;
    Out.Ln
  END;

  (* "First the case expression is evaluated": once per statement, however
     many alternatives its value has to be compared against. *)
  calls := 0;
  CASE Selector(3) OF
    1: Out.Int(1, 4)
  | 2: Out.Int(2, 4)
  | 3: Out.Int(3, 4)
  END;
  Out.Int(calls, 4); Out.Ln
END Case.
