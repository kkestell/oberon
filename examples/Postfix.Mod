(*  Reads one expression per line from standard input and stops at a blank
    line.

    Source: AntKrotov/oberon-07-compiler (BSD-2-Clause),
    samples/windows-console/postfix.ob07, adapted to Oberon-07 by 0CodErr of
    the KolibriOS team.

    Changed: Dropped the KolibriOS Console module. Reads with In.Line rather
    than In.String, which in this library reads a quoted string. Renamed the
    identifiers holding an underscore, which Oberon-07 does not allow.
*)

(* Example program from Programming In Modula-2, N. Wirth., pg. 56, *)
(*  - no WINDOWS in this example *)

(* this program translates a small language into postfix form
 *  the language is
 *
 *           expression = term { [ "+" | "-" ] term }
 *
 *           term = factor { [ "*" | "/" ] factor }
 *
 *           factor = letter | "(" expression ")"
 *
 *           letter  =  "a" | 'b" | … | "z"
 *
 *   try as input
 *      a+b
 *      a*b+c
 *      a+b*c
 *      a*(b/(c-d))
 *)

MODULE Postfix;

IMPORT In, Out;


CONST
    OutLineSize = 80;
    InLineSize = 80;


VAR
    ch       : CHAR;
    i, index : INTEGER;
    outLine : ARRAY OutLineSize OF CHAR;
    inLine  : ARRAY InLineSize OF CHAR;
    curCh   : INTEGER;


PROCEDURE NextChar(): CHAR;
BEGIN
    INC(curCh)
    RETURN inLine[curCh - 1]
END NextChar;


PROCEDURE expression;
VAR
    addop :CHAR;


    PROCEDURE term;
    VAR
        mulop :CHAR;


        PROCEDURE factor;
        BEGIN (* factor *)
            IF ch = "(" THEN
                ch := NextChar();
                expression;
                WHILE ch # ")" DO
                    ch := NextChar()
                END (* WHILE *)
            ELSE
                WHILE (ch < "a") OR (ch > "z") DO
                    ch := NextChar()
                END; (* WHILE *)
                outLine[index] := ch;
                index := index + 1
            END; (* IF *)
            ch := NextChar()
        END factor;


    BEGIN (* term *)
        factor;
        WHILE (ch = "*") OR (ch = "/") DO
            mulop := ch;
            ch := NextChar();
            factor;
            outLine[index] := mulop;
            index := index + 1
        END (* WHILE *)
    END term;


BEGIN (* expression *)
    term;
    WHILE (ch = "+") OR (ch = "-") DO
        addop := ch;
        ch := NextChar();
        term;
        outLine[index] := addop;
        index := index + 1
    END (* WHILE *)
END expression;


BEGIN (* Postfix *)

    index := 1; curCh := 0;
    Out.String("Enter expression: ");
    inLine := ""; In.Line(inLine);
    ch := NextChar();
    WHILE ch > " " DO
        expression;
        FOR i := 1 TO index - 1 DO
            Out.Char(outLine[i])
        END; (* FOR *)
        Out.Ln;
        index := 1; curCh := 0;
        Out.String("Enter expression: ");
        inLine := ""; In.Line(inLine);
        ch := NextChar()
    END (* WHILE *)
END Postfix.
