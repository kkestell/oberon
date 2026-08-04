(*  Out.String writes bytes, so the UTF-8 source text reaches the terminal
    unchanged.

    Source: lboasso/oberonc (MIT), examples/Hello.Mod.
*)

MODULE Hello;
  IMPORT Out; (* Import Out to print on the console *)
BEGIN
  Out.String("Hello 世界");
  Out.Ln (* print a new line *)
END Hello.
