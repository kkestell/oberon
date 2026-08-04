(*  The Sierpinski triangle in text. Reads the order from standard input.

    Source: AntKrotov/oberon-07-compiler (BSD-2-Clause),
    samples/windows-console/SierpinskiTriangle.ob07, adapted to Oberon-07 by
    0CodErr of the KolibriOS team.

    Changed: Dropped the KolibriOS Console module and the closing keypress
    pause. Replaced BITS, which Oberon-07 does not define, with a Bits
    procedure that builds the same SET from an INTEGER.
*)

MODULE SierpinskiTriangle;

IMPORT In, Out;


VAR
    order: INTEGER;


(* Oberon-07 has no INTEGER-to-SET conversion, so build the bit set by hand. *)
PROCEDURE Bits(n: INTEGER): SET;
VAR
    i:  INTEGER;
    bs: SET;

BEGIN
    bs := {};
    i := 0;
    WHILE n # 0 DO
        IF ODD(n) THEN INCL(bs, i) END;
        n := n DIV 2;
        INC(i)
    END

    RETURN bs
END Bits;


PROCEDURE PrintSierpinski(order: INTEGER);
VAR
    x, y, k, size: INTEGER;

BEGIN
    size := LSL(1, order) - 1;
    FOR y := size TO 0 BY -1 DO
        FOR k := 1 TO y DO
            Out.Char(" ")
        END;
        FOR x := 0 TO size - y DO
            IF Bits(x) * Bits(y) = {} THEN
                Out.String("* ")
            ELSE
                Out.String("  ")
            END
        END;
        Out.Ln
    END
END PrintSierpinski;


BEGIN

    Out.String("Input triangle order(0..5):");
    In.Int(order); Out.Ln;
    PrintSierpinski(order)
END SierpinskiTriangle.
