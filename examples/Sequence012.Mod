(*  Source: AntKrotov/oberon-07-compiler (BSD-2-Clause),
    samples/windows-console/sequence012.ob07, adapted to Oberon-07 by 0CodErr
    of the KolibriOS team.

    Changed: Dropped the KolibriOS Console module and the closing keypress
    pause.
*)

(* Find sequence of digits 0, 1, 2 and of lengths 1 ... 90, such
   that they contain no two adjacent subsequences that are equal *)

MODULE Sequence012;

IMPORT Out;


CONST
    maxlength = 75;


VAR
    n: INTEGER;
    good: BOOLEAN;
    s: ARRAY maxlength OF INTEGER;


PROCEDURE printsequence;
VAR
    k: INTEGER;
BEGIN
    Out.Char(" ");
    FOR k := 1 TO n DO Out.Int(s[k], 1) END;
    Out.Ln
END printsequence;


PROCEDURE changesequence;
BEGIN
    IF s[n] = 3 THEN
        DEC(n);
        changesequence
    ELSE
        s[n] := s[n] + 1
    END
END changesequence;


PROCEDURE try;
VAR
    i, l, nhalf: INTEGER;

BEGIN
    IF n <= 1 THEN
        good := TRUE
    ELSE
        l := 0; nhalf := n DIV 2;
        REPEAT
            INC(l); i := 0;
            REPEAT
                good := s[n - i] # s[n - l - i];
                INC(i)
            UNTIL good OR (i = l)
        UNTIL ~good OR (l >= nhalf)
    END
END try;


BEGIN

    n := 0;
    REPEAT
        INC(n);
        s[n] := 1; try;
        WHILE ~good DO
            changesequence;
            try
        END;
        printsequence
    UNTIL n >= maxlength - 1
END Sequence012.
