(*  The Sierpinski carpet in text. Reads the order from standard input.

    Source: AntKrotov/oberon-07-compiler (BSD-2-Clause),
    samples/windows-console/SierpinskiCarpet.ob07, adapted to Oberon-07 by
    0CodErr of the KolibriOS team.

    Changed: Dropped the KolibriOS Console module and the closing keypress
    pause. Renamed in_carpet, since an Oberon-07 identifier holds no
    underscore.
*)

MODULE SierpinskiCarpet;

IMPORT In, Out;


VAR
    order: INTEGER;


PROCEDURE pow(b, n: INTEGER): INTEGER;
VAR
    i, res: INTEGER;

BEGIN
    res := 1;
    FOR i := 1 TO n DO
        res := res * b
    END

    RETURN res
END pow;


PROCEDURE InCarpet(x, y: INTEGER): BOOLEAN;
VAR
    res, exit: BOOLEAN;

BEGIN
    exit := FALSE;
    res := TRUE;
    WHILE (x > 0) & (y > 0) & (exit = FALSE) DO
        IF (x MOD 3 = 1) & (y MOD 3 = 1) THEN
            res := FALSE;
            exit := TRUE
        END;
        y := y DIV 3;
        x := x DIV 3
    END

    RETURN res
END InCarpet;


PROCEDURE PrintSierpinski(n: INTEGER);
VAR
    i, j, l: INTEGER;

BEGIN
    l := pow(3, n) - 1;
    FOR i := 0 TO l DO
        FOR j := 0 TO l DO
            IF InCarpet(i, j) THEN
                Out.Char("#")
            ELSE
                Out.Char(" ")
            END
        END;
        Out.Ln
    END
END PrintSierpinski;


BEGIN

    Out.String("Input carpet order(0..3):");
    In.Int(order); Out.Ln;
    PrintSierpinski(order)
END SierpinskiCarpet.
