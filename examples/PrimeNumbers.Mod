(*  Sieves each candidate against a running multiple of every smaller prime.
    Reads how many primes to print from standard input.

    Source: lboasso/oberonc (MIT), examples/PrimeNumbers/PrimeNumbers.Mod,
    after Wirth.

    Changed: Replaced ARGNUM and ARGS, which are oberonc extensions, with
    In.Int, which also removes the need for the Util module. The original left
    p and v undefined and relied on the JVM zeroing them, so both are now
    seeded explicitly and the table starts at 2 rather than 5.
*)

MODULE PrimeNumbers;  (*Tabulate prime numbers*)
  IMPORT In, Out;

  PROCEDURE Primes(n: INTEGER);
    VAR i, k, m, x, inc, lim, sqr: INTEGER;
      prim: BOOLEAN;
      p: ARRAY 400 OF INTEGER;
      v: ARRAY 20 OF INTEGER;
  BEGIN
    (* p[k] is the k-th prime and v[k] a running multiple of it. Oberon-07
       leaves a local undefined until it is assigned, so seed both here. *)
    FOR i := 0 TO LEN(v) - 1 DO v[i] := 0 END ;
    p[1] := 2; p[2] := 3;
    Out.Int(p[1], 5); Out.Int(p[2], 5);
    x := 1; inc := 4;
    lim := 1; sqr := 4;
     m := 2; i := 3;
    WHILE i <= n DO
      REPEAT x := x + inc; inc := 6 - inc;
        IF sqr <= x THEN  (*sqr = p[lim]^2*)
          v[lim] := sqr; lim := lim + 1; sqr := p[lim]*p[lim]
        END ;
        k := 2; prim := TRUE;
        WHILE prim & (k < lim) DO
          k := k+1;
          IF v[k] < x THEN v[k] := v[k] + p[k] END ;
          prim := x # v[k]
        END
      UNTIL prim;
      p[i] := x; Out.Int(x, 5);
      IF m = 10 THEN Out.Ln; m := 0 ELSE m := m+1 END ;
      i := i+1
    END ;
    IF m > 0 THEN Out.Ln END
  END Primes;

  PROCEDURE Main;
    VAR n: INTEGER;
  BEGIN
    Out.String("How many primes to tabulate [2..99]? "); In.Int(n); Out.Ln;
    IF In.Done & (n >= 2) & (n <= 99) THEN Primes(n)
    ELSE Out.String("Expected a number in [2..99]"); Out.Ln
    END
  END Main;

BEGIN Main
END PrimeNumbers.
