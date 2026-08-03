MODULE OutAlias;
  (* The native Out is installed through the same alias path as a source
     module's interface. *)
  IMPORT O := Out;
BEGIN
  O.Int(7, 0); O.Ln
END OutAlias.
