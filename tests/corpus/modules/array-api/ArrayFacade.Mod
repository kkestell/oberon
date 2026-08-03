MODULE ArrayFacade;
  (* A re-exported alias. Report 6.2 identity is carried by the type itself,
     not by the name it is reached through, so ArrayFacade.Row and
     ArrayTypes.Row are one type. *)
  IMPORT ArrayTypes;
  TYPE Row* = ArrayTypes.Row;
END ArrayFacade.
