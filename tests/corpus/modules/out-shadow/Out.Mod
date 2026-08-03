MODULE Out;
  (* A source module named Out, next to the root that imports it. It replaces
     the temporary native Out for this build, so Int and Ln are gone and
     Silence is what the client can call. *)
  PROCEDURE Silence*;
  END Silence;

END Out.
