(*  Reads the order of the square from standard input.

    Source: AntKrotov/oberon-07-compiler (BSD-2-Clause),
    samples/windows-console/MagicSquares.ob07, adapted to Oberon-07 by 0CodErr
    of the KolibriOS team.

    Changed: Dropped the KolibriOS Console module and the closing keypress
    pause. Dropped two leftover debugging writes from inside the generator
    loop.
*)

(* ********* Zonnon online collection ***********
 * Magic Squares
 *
 * This example is a part of Prof. Nikalus Wirth's book
 * www.zonnon.ethz.ch/usergroup
 * (c) ETH Zurich
 *)

MODULE MagicSquares;   (*NW 11.8.97*)

IMPORT In, Out;


PROCEDURE Generate;  (*magic square of order 3, 5, 7, ... *)
VAR
    i, j, x, nx, nsq, n: INTEGER;
    M: ARRAY 13, 13 OF INTEGER;

BEGIN
    Out.String("Enter magic square order(3, 5, 7, ..., 13): "); In.Int(n); Out.Ln; nsq := n * n; x := 0;
    i := n DIV 2; j := n - 1;
    WHILE x < nsq DO
        nx := n + x; j := (j - 1) MOD n; INC(x);
        M[i, j] := x;
        WHILE x < nx DO
            i := (i + 1) MOD n; j := (j + 1) MOD n;
            INC(x); M[i, j] := x
        END
    END;
    FOR i := 0 TO n - 1 DO
        FOR j := 0 TO n - 1 DO Out.Int(M[i, j], 6) END;
        Out.Ln
    END
END Generate;

BEGIN

    Generate
END MagicSquares.
