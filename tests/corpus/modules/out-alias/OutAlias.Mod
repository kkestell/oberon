MODULE OutAlias;
  (* A bundled source module follows the ordinary import-alias path. *)
  IMPORT O := Out;
BEGIN
  O.Int(7, 0); O.Ln
END OutAlias.
