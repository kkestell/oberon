MODULE ArrayProgramStorage;
  (* Each module's own globals fit the target limit. The three of them
     together do not, and the build stops before QBE sees any of it. *)
  IMPORT ArrayBlockOne, ArrayBlockTwo;
  VAR block: ARRAY 100000000 OF INTEGER;
BEGIN
  block[0] := ArrayBlockOne.block[0] + ArrayBlockTwo.block[0]
END ArrayProgramStorage.
