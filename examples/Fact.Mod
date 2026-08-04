(*  Source: AntKrotov/oberon-07-compiler (BSD-2-Clause),
    samples/windows-console/fact.ob07, adapted to Oberon-07 by 0CodErr of the
    KolibriOS team.

    Changed: Dropped the KolibriOS Console module and the closing keypress
    pause. Renamed MAX_INTEGER, since an Oberon-07 identifier holds no
    underscore.
*)

(*
 * Written by Andrew Cadach
 *
 * Recursive (extremely uneficient:-) implementation of factorial
 *
 *                     n * (n-1)!, n <> 0
 * By definition, n! =
 *                     1, n = 0
 *
 *)

MODULE Fact;

IMPORT Out;


CONST
    MaxInteger = ROR(-2, 1);


VAR
    i, r: INTEGER;


PROCEDURE f (n: INTEGER): INTEGER;
VAR
    Res: INTEGER;

BEGIN
    IF n = 0 THEN
        Res := 1
    ELSE
        Res := n * f (n - 1)
    END

    RETURN Res
END f;


BEGIN

    i := 0;
    REPEAT
        r := f(i);
        Out.String ("The factorial of ");
        Out.Int (i, 2);
        Out.String (" is ");
        Out.Int (r, 0);
        Out.Ln;
        INC(i)
    UNTIL r >= MaxInteger DIV i
END Fact.
