MODULE ArrayPrivateType;
  (* An unmarked type is absent from the interface, so the client cannot name
     it even though the module it lives in is imported. *)
  IMPORT ArrayHidden;
  VAR fine: ArrayHidden.Shown;
      wrong: ArrayHidden.Hidden;
BEGIN
  fine[0] := 1
END ArrayPrivateType.
