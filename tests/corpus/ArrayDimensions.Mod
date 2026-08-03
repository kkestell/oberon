MODULE ArrayDimensions;
  (* Report 6.2 makes ARRAY 2, 3 OF INTEGER mean ARRAY 2 OF ARRAY 3 OF
     INTEGER, so the two spellings below have the same shape and are filled
     and read by the same code. Report 8.1 makes g[i, j] mean g[i][j], so the
     index expressions run left to right and exactly once; Tick numbers its
     own calls, which is how that order is visible in the output. *)
  IMPORT Out;
  VAR comma: ARRAY 2, 3 OF INTEGER;
      nested: ARRAY 2 OF ARRAY 3 OF INTEGER;
      i, j, calls: INTEGER;

  PROCEDURE Tick(): INTEGER;
  BEGIN INC(calls)
  RETURN calls
  END Tick;

  PROCEDURE Inner;
    (* LEN in a required constant context: a constant declaration and an array
       length. Both selectors are constant and in range, so these fold and
       emit no code at all, while the LEN in the loop below is an ordinary
       executable call that still resolves its designator. *)
    CONST Width = LEN(comma[0]);
    TYPE Small = ARRAY 2 OF INTEGER;
    VAR copy: ARRAY LEN(comma[1]) OF INTEGER;
        k: INTEGER;

    PROCEDURE Deeper;
      (* Report 10: a nested procedure sees the enclosing procedure's types
         even though it cannot see its variables. *)
      VAR own: Small;
    BEGIN
      own[0] := 7; own[1] := own[0] + 1;
      Out.Int(LEN(own), 4); Out.Int(own[1], 4); Out.Ln
    END Deeper;

  BEGIN
    FOR k := 0 TO Width - 1 DO copy[k] := comma[1, k] * 2 END;
    Out.Int(Width, 4); Out.Int(LEN(copy), 4);
    FOR k := 0 TO LEN(copy) - 1 DO Out.Int(copy[k], 4) END;
    Out.Ln;
    Deeper
  END Inner;

BEGIN
  FOR i := 0 TO LEN(comma) - 1 DO
    FOR j := 0 TO LEN(comma[i]) - 1 DO
      comma[i, j] := i * 10 + j;
      nested[i][j] := comma[i, j] + 100
    END
  END;
  Out.Int(comma[0, 0], 4); Out.Int(comma[1, 2], 4);
  Out.Int(nested[0][0], 4); Out.Int(nested[1][2], 4); Out.Ln;
  Out.Int(LEN(comma), 4); Out.Int(LEN(comma[0]), 4);
  Out.Int(LEN(nested), 4); Out.Int(LEN(nested[1]), 4); Out.Ln;

  (* Two index expressions in one bracket list. The first call returns 1 and
     the second returns 2. Left to right that reads comma[0, 1], which holds
     1; right to left it would read comma[1, 0], which holds 10. The call
     count then shows each expression ran exactly once. *)
  calls := 0;
  Out.Int(comma[Tick() - 1, Tick() - 1], 4); Out.Int(calls, 4); Out.Ln;

  (* The argument of LEN is evaluated even though the answer is static. *)
  Out.Int(LEN(nested[Tick() - 3]), 4); Out.Int(calls, 4); Out.Ln;
  Inner
END ArrayDimensions.
