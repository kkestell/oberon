(*  Reads the width and the height from standard input.

    Source: AntKrotov/oberon-07-compiler (BSD-2-Clause),
    samples/windows-console/SpiralMatrix.ob07, adapted to Oberon-07 by 0CodErr
    of the KolibriOS team.

    Changed: Dropped the KolibriOS Console module and the closing keypress
    pause. Renamed print_spiral, since an Oberon-07 identifier holds no
    underscore.
*)

(*
   Produce a spiral array.
   A spiral array is a square arrangement of the first (Width * Height) natural numbers,
   where the numbers increase sequentially as you go around the edges of the array spiraling inwards.
*)

MODULE SpiralMatrix;

IMPORT In, Out;


VAR
    Width, Height: INTEGER;


PROCEDURE spiral(w, h, x, y: INTEGER): INTEGER;
VAR
    res: INTEGER;

BEGIN
    IF y # 0 THEN
        res := w + spiral(h - 1, w, y - 1, w - x - 1)
    ELSE
        res := x
    END
    RETURN res
END spiral;


PROCEDURE PrintSpiral(w, h: INTEGER);
VAR
    i, j: INTEGER;

BEGIN
    FOR i := 0 TO h - 1 DO
        FOR j := 0 TO w - 1 DO
            Out.Int(spiral(w, h, j, i), 4)
        END;
        Out.Ln
    END
END PrintSpiral;


BEGIN

    Out.String("Input width of matrix(1, 2, 3, ...):");  In.Int(Width);
    Out.String("Input height of matrix:(1, 2, 3, ...)"); In.Int(Height); Out.Ln;
    PrintSpiral(Width, Height)
END SpiralMatrix.
