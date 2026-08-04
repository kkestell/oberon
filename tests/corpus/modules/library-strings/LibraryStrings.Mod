MODULE LibraryStrings;

IMPORT Strings, Out;

VAR s: ARRAY 16 OF CHAR; short: ARRAY 4 OF CHAR; one: ARRAY 1 OF CHAR; full, pattern: ARRAY 3 OF CHAR;

BEGIN
  full[0] := "a"; full[1] := "b"; full[2] := "c";
  pattern[0] := "b"; pattern[1] := "c"; pattern[2] := "d";
  ASSERT(Strings.Length(full) = 3);
  ASSERT(Strings.Pos(pattern, full, 0) = -1);
  ASSERT(Strings.Pos("", full, 3) = 3);
  s := "cde"; Strings.Insert("ab", 0, s); ASSERT(s = "abcde");
  s := "abc"; Strings.Insert(s, 1, s); ASSERT(s = "aabcbc");
  short := "ab"; Strings.Append(short, short); ASSERT(short = "aba");
  s := "abcdef"; Strings.Delete(s, 2, 2); ASSERT(s = "abef");
  Strings.Delete(s, 4, 99); ASSERT(s = "abef");
  Strings.Delete(s, 0, 0); ASSERT(s = "abef");
  s := "abcdef"; Strings.Replace(s, 2, s); ASSERT(s = "ababcdef");
  s := "abcdef"; Strings.Extract(s, 2, 3, s); ASSERT(s = "cde");
  Strings.Extract("abcdef", 1, 0, one); ASSERT(one = "");
  Strings.Extract("abcdef", 1, 5, short); ASSERT(short = "bcd");
  ASSERT(Strings.Pos("bc", "abcabc", 2) = 4);
  s := "az AZ 9"; Strings.Cap(s); ASSERT(s = "AZ AZ 9");
  Out.String("ok"); Out.Ln
END LibraryStrings.
