MODULE CharBad;
  VAR ch: CHAR; i: INTEGER; a: ARRAY 4 OF CHAR; b: BOOLEAN;
BEGIN
  ch := ch + ch;
  i := ch;
  ch := i;
  i := a[ch];
  b := ch = a;
  i := ORD();
  i := ORD(1);
  ch := CHR("a");
  ch := CHR(1, 2);
  ch := CHR(256);
  ch := CHR(-1)
END CharBad.
