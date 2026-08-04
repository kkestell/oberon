(*  Source: AntKrotov/oberon-07-compiler (BSD-2-Clause),
    samples/windows-console/hailst.ob07, adapted to Oberon-07 by 0CodErr of
    the KolibriOS team.

    Changed: Dropped the KolibriOS Console module and the closing keypress
    pause. The overflow exit now uses ASSERT, since Oberon-07 has no HALT and
    this compiler offers no process-exit primitive. Renamed the parameter
    _type, since an Oberon-07 identifier holds no underscore.
*)

(*
   The Hailstone sequence of numbers can be generated
   from a starting positive integer, n by:
     IF n is   1  THEN the sequence ends.
     IF n is even THEN the next n of the sequence = n / 2
     IF n is odd  THEN the next n of the sequence = (3 * n) + 1
   The (unproven) Collatz conjecture is that the hailstone sequence
   for any starting number always terminates.
*)

MODULE Hailstone;

IMPORT Out;


CONST
    maxCard = ROR(-2, 1) DIV 3;
    List    = 1;
    Count   = 2;
    Max     = 3;


VAR
    a: INTEGER;


PROCEDURE HailStone(start, kind: INTEGER): INTEGER;
VAR
    n, max, count, res: INTEGER;
    exit: BOOLEAN;

BEGIN
    count := 1;
    n := start;
    max := n;
    exit := FALSE;
    WHILE exit # TRUE DO
        IF kind = List THEN
            Out.Int (n, 12);
            IF count MOD 6 = 0 THEN Out.Ln END
        END;
        IF n # 1 THEN
            IF ODD(n) THEN
                IF n < maxCard THEN
                    n := 3 * n + 1;
                    IF n > max THEN max := n END
                ELSE
                    Out.String("Exceeding max value for type INTEGER at:");
                    Out.Ln;
                    Out.String("n = ");       Out.Int(start, 1);
                    Out.String(", count = "); Out.Int(count, 1);
                    Out.String(", intermediate value ");
                    Out.Int(n, 1);
                    Out.String(". Aborting.");
                    Out.Ln;
                    ASSERT(FALSE)
                END
            ELSE
                n := n DIV 2
            END;
            INC(count)
        ELSE
            exit := TRUE
        END
    END;
    IF kind = Max THEN res := max ELSE res := count END

    RETURN res
END HailStone;


PROCEDURE FindMax(num: INTEGER);
VAR
    val, maxCount, maxVal, cnt: INTEGER;

BEGIN
    maxCount := 0;
    maxVal := 0;
    FOR val := 2 TO num DO
        cnt := HailStone(val, Count);
        IF cnt > maxCount THEN
            maxVal := val;
            maxCount := cnt
        END
    END;
    Out.String("Longest sequence below "); Out.Int(num, 1);
    Out.String(" is ");                    Out.Int(HailStone(maxVal, Count), 1);
    Out.String(" for n = ");               Out.Int(maxVal, 1);
    Out.String(" with an intermediate maximum of ");
    Out.Int(HailStone(maxVal, Max), 1);
    Out.Ln
END FindMax;


BEGIN

    a := HailStone(27, List);
    Out.Ln;
    Out.String("Iterations total = "); Out.Int(HailStone(27, Count), 1);
    Out.String(" max value = ");       Out.Int(HailStone(27, Max), 1);
    Out.Ln;
    FindMax(100000);
    Out.String("Done.");
    Out.Ln
END Hailstone.
