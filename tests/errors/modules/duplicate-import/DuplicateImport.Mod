MODULE DuplicateImport;
  (* Report 4: one identifier, one object per scope. The repeated qualifier,
     the reused alias, and the later declaration all collide in the module
     scope. Importing one module under two distinct qualifiers is legal and
     is covered by the Api corpus module. *)
  IMPORT M, M, X := M, X := N;
  VAR X: INTEGER;
END DuplicateImport.
