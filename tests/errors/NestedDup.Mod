MODULE NestedDup;
PROCEDURE Outer;
  PROCEDURE Inner(x: INTEGER);
  VAR x: INTEGER;
  END Inner;
  PROCEDURE x;
  END x;
  PROCEDURE x;
  END x;
END Outer;
END NestedDup.
