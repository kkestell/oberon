(*  Source: AntKrotov/oberon-07-compiler (BSD-2-Clause),
    samples/windows-console/HeapSort.ob07, adapted to Oberon-07 by 0CodErr of
    the KolibriOS team.

    Changed: Dropped the KolibriOS Console module and the closing keypress
    pause. Renamed MAX_SIZE, since an Oberon-07 identifier holds no
    underscore.
*)

(* ********* Zonnon online collection ***********
 * Sorting: Heap Sort (Chapter 2, Example 2.8)
 *
 * This example is a part of Prof. Nikalus Wirth's book
 * www.zonnon.ethz.ch/usergroup
 * (c) ETH Zurich
 *)

MODULE HeapSort;

IMPORT Out;


CONST
    MaxSize = 20;


TYPE
    DefaultArray = ARRAY MaxSize OF INTEGER;


VAR
    MyArray: DefaultArray;

 (***** Implementation *****)

PROCEDURE sift(VAR a: DefaultArray; L,R:INTEGER);
VAR
    i, j, x: INTEGER;

BEGIN
    i := L; j:= 2 * L; x:= a[L];
    IF (j < R) & (a[j] < a[j + 1]) THEN j := j + 1 END;
    WHILE (j <= R) & (x < a[j]) DO
        a[i] := a[j]; i := j; j := 2 * j;
        IF (j < R) & (a[j] < a[j + 1]) THEN j := j + 1 END
    END;
    a[i] := x
END sift;


PROCEDURE HeapSort(VAR a: DefaultArray; n: INTEGER);
VAR
    L, R, x: INTEGER;

BEGIN
    L := (n DIV 2); R := n - 1;
    WHILE L > 0 DO L := L - 1; sift(a, L, R) END;
    WHILE R > 0 DO
        x := a[0]; a[0] := a[R]; a[R]:= x;
        R := R - 1; sift(a, L, R)
    END
END HeapSort;

(***** Example support *****)

PROCEDURE FillTheArray;
VAR
    i: INTEGER;

BEGIN
    FOR i := 0 TO MaxSize - 1 DO
        MyArray[i] := ABS(10 - i)
    END
END FillTheArray;


PROCEDURE PrintTheArray;
VAR
    i: INTEGER;

BEGIN
    Out.String("Array:"); Out.Ln;
    FOR i := 0 TO MaxSize - 1 DO
        Out.Int(MyArray[i], 2); Out.String(", ")
    END;
    Out.Ln
END PrintTheArray;


PROCEDURE Execute;
BEGIN
    HeapSort(MyArray, MaxSize)
END Execute;


BEGIN

    Out.String("Example 2.8 (Heap sort)"); Out.Ln;
    FillTheArray;
    PrintTheArray;
    Execute;
    PrintTheArray
END HeapSort.
