MODULE Strings;

PROCEDURE Min(a, b: INTEGER): INTEGER;
BEGIN
  IF b < a THEN a := b END
RETURN a
END Min;

PROCEDURE Length*(s: ARRAY OF CHAR): INTEGER;
  VAR i: INTEGER;
BEGIN
  i := 0;
  WHILE (i < LEN(s)) & (s[i] # 0X) DO INC(i) END
RETURN i
END Length;

PROCEDURE Terminated(s: ARRAY OF CHAR): BOOLEAN;
RETURN Length(s) < LEN(s)
END Terminated;

PROCEDURE Insert*(source: ARRAY OF CHAR; pos: INTEGER; VAR dest: ARRAY OF CHAR);
  VAR sourceLength, destLength, newLength, i: INTEGER;
BEGIN
  sourceLength := Length(source);
  destLength := Length(dest);
  ASSERT(sourceLength < LEN(source));
  ASSERT(destLength < LEN(dest));
  ASSERT(LEN(dest) > 0);
  ASSERT(pos >= 0);
  ASSERT(pos <= destLength);
  newLength := Min(destLength + sourceLength, LEN(dest) - 1);
  i := newLength - 1;
  WHILE i >= pos DO
    IF i < pos + sourceLength THEN
      dest[i] := source[i - pos]
    ELSE
      dest[i] := dest[i - sourceLength]
    END;
    DEC(i)
  END;
  dest[newLength] := 0X
END Insert;

PROCEDURE Append*(extra: ARRAY OF CHAR; VAR dest: ARRAY OF CHAR);
  VAR pos: INTEGER;
BEGIN
  ASSERT(Terminated(extra));
  pos := Length(dest);
  ASSERT(pos < LEN(dest));
  ASSERT(LEN(dest) > 0);
  Insert(extra, pos, dest)
END Append;

PROCEDURE Delete*(VAR s: ARRAY OF CHAR; pos, n: INTEGER);
  VAR length, count, i: INTEGER;
BEGIN
  length := Length(s);
  ASSERT(length < LEN(s));
  ASSERT(pos >= 0);
  ASSERT(pos <= length);
  ASSERT(n >= 0);
  count := Min(n, length - pos);
  i := pos;
  WHILE i <= length - count DO
    s[i] := s[i + count];
    INC(i)
  END
END Delete;

PROCEDURE Replace*(source: ARRAY OF CHAR; pos: INTEGER; VAR dest: ARRAY OF CHAR);
  VAR sourceLength, destLength, count, newLength, i: INTEGER;
BEGIN
  sourceLength := Length(source);
  destLength := Length(dest);
  ASSERT(sourceLength < LEN(source));
  ASSERT(destLength < LEN(dest));
  ASSERT(LEN(dest) > 0);
  ASSERT(pos >= 0);
  ASSERT(pos <= destLength);
  count := Min(sourceLength, LEN(dest) - 1 - pos);
  i := count - 1;
  WHILE i >= 0 DO
    dest[pos + i] := source[i];
    DEC(i)
  END;
  newLength := destLength;
  IF pos + count > newLength THEN newLength := pos + count END;
  dest[newLength] := 0X
END Replace;

PROCEDURE Extract*(source: ARRAY OF CHAR; pos, n: INTEGER; VAR dest: ARRAY OF CHAR);
  VAR sourceLength, count, i: INTEGER;
BEGIN
  sourceLength := Length(source);
  ASSERT(sourceLength < LEN(source));
  ASSERT(LEN(dest) > 0);
  ASSERT(pos >= 0);
  ASSERT(pos <= sourceLength);
  ASSERT(n >= 0);
  count := Min(n, Min(sourceLength - pos, LEN(dest) - 1));
  i := 0;
  WHILE i < count DO
    dest[i] := source[pos + i];
    INC(i)
  END;
  dest[count] := 0X
END Extract;

PROCEDURE Pos*(pattern, s: ARRAY OF CHAR; pos: INTEGER): INTEGER;
  VAR patternLength, stringLength, candidate, i, result: INTEGER;
BEGIN
  patternLength := Length(pattern);
  stringLength := Length(s);
  ASSERT(pos >= 0);
  ASSERT(pos <= stringLength);
  result := -1;
  candidate := pos;
  WHILE (result = -1) & (candidate <= stringLength - patternLength) DO
    i := 0;
    WHILE (i < patternLength) & (pattern[i] = s[candidate + i]) DO INC(i) END;
    IF i = patternLength THEN result := candidate ELSE INC(candidate) END
  END
RETURN result
END Pos;

PROCEDURE Cap*(VAR s: ARRAY OF CHAR);
  VAR length, i: INTEGER;
BEGIN
  length := Length(s);
  ASSERT(length < LEN(s));
  i := 0;
  WHILE i < length DO
    IF (s[i] >= "a") & (s[i] <= "z") THEN
      s[i] := CHR(ORD(s[i]) - ORD("a") + ORD("A"))
    END;
    INC(i)
  END
END Cap;

END Strings.
