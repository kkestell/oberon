MODULE SetRanges;
  (* Report 8.2: {m .. n} is empty when m > n. That rule chooses the value,
     it does not skip the work: both endpoints are evaluated exactly once,
     left to right, whichever way round they are. Each endpoint procedure
     counts its calls and appends its own digit to `order`. *)
  IMPORT Out;
  VAR calls, order: INTEGER; s: SET;

  PROCEDURE Low(n: INTEGER): INTEGER;
  BEGIN
    INC(calls); order := order * 10 + 1
  RETURN n
  END Low;

  PROCEDURE High(n: INTEGER): INTEGER;
  BEGIN
    INC(calls); order := order * 10 + 2
  RETURN n
  END High;

  PROCEDURE Report;
  BEGIN
    Out.Int(ORD(s), 14); Out.Int(calls, 4); Out.Int(order, 6); Out.Ln;
    calls := 0; order := 0
  END Report;

BEGIN
  calls := 0; order := 0;
  s := {Low(2) .. High(5)}; Report;
  s := {Low(5) .. High(2)}; Report;
  s := {Low(0), Low(3) .. High(4), High(31)}; Report
END SetRanges.
