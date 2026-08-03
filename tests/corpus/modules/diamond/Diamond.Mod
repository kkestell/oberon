MODULE Diamond;
  (* Left and Right both import Leaf, and the root imports Left first. The
     digits come out in initialization order, and Leaf's digit appears once. *)
  IMPORT Out, Left, Right;
BEGIN
  Out.Int(4, 0); Out.Ln
END Diamond.
