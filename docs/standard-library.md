# Standard Library

The bundled library contains `Out`, `In`, `Math`, `Strings`, and `Files`. An application may replace a bundled module by placing a module with the same exact file name in the root source directory. The character set is the compiler's byte character set: a `CHAR` is an ordinal from 0 through 255, quoted source text contributes its encoded bytes, and the library does not decode Unicode or apply a locale.

## `Out`

```oberon
PROCEDURE Open;
PROCEDURE Char(ch: CHAR);
PROCEDURE String(s: ARRAY OF CHAR);
PROCEDURE Int(i, n: INTEGER);
PROCEDURE Hex(i: INTEGER);
PROCEDURE Real(x: REAL; n: INTEGER);
PROCEDURE Ln;
```

`Open` does nothing. `Char` writes one byte. `String` writes bytes until the first null or the actual array bound, so a full unterminated array is safe and is written in full. `Ln` writes one line-feed byte.

`Int` writes decimal without a leading plus and pads on the left with spaces to the positive minimum width `n`; zero and negative widths add no padding. Every 32-bit `INTEGER`, including `MIN(INTEGER)`, is supported. `Hex` writes one space and eight uppercase hexadecimal digits containing the integer's 32-bit pattern.

`Real` writes a binary32 value in uppercase exponential notation with six fractional digits and a signed exponent of at least two digits. The mantissa has no leading plus. A positive `n` is a minimum width padded on the left with spaces; zero and negative widths add no padding. On the supported C target, infinities and NaNs use the target's uppercase `INF` and `NAN` spellings and negative zero retains its sign.

## `In`

```oberon
VAR Done: BOOLEAN;
PROCEDURE Open;
PROCEDURE Char(VAR ch: CHAR);
PROCEDURE Int(VAR i: INTEGER);
PROCEDURE Real(VAR x: REAL);
PROCEDURE String(VAR str: ARRAY OF CHAR);
PROCEDURE Name(VAR name: ARRAY OF CHAR);
PROCEDURE Line(VAR line: ARRAY OF CHAR);
```

Clients may read `Done` but cannot assign it. Every public operation sets it. `Open` does not rewind standard input and succeeds only before an input operation has consumed a byte. `Char` reads the next byte without skipping whitespace; at end of file it fails and leaves its destination unchanged.

`Int`, `Real`, `String`, and `Name` skip ASCII space, tab, line feed, carriage return, form feed, and vertical tab. Their classifications do not depend on the process locale. A scalar destination changes only after a complete valid token has been read. A malformed or overflowing token is consumed through its token boundary. `Int`, `Real`, and `String` leave terminating whitespace current; `Name` consumes the byte that ends its graphical sequence.

`Int` accepts an optional minus sign followed by decimal digits, or one through eight uppercase hexadecimal digits followed by `H`. An eight-digit hexadecimal token supplies the corresponding 32-bit pattern. `Real` accepts an optional sign, one or more decimal digits, a decimal point, zero or more decimal digits, and an optional `E` exponent with an optional sign and one or more digits. Conversion rounds once to binary32; malformed tokens and non-finite results fail.

`String` accepts bytes between double quotes, including spaces but not a line feed, or a hexadecimal ordinal from 0 through 255 followed by `X`. `Name` accepts one nonempty sequence of graphical non-whitespace bytes. A successful array read always appends a null byte. A zero-length destination is untouched and fails. A one-byte destination can hold only the empty quoted string. When a nonempty `String` or `Name` result does not fit, the destination becomes an empty terminated string and `Done` is false.

`Line` begins at the current position, consumes through line feed or end of file, and does not skip whitespace. An empty line is a successful empty string. A result that does not fit is truncated to a terminated prefix, the rest of the line is consumed, and `Done` is false. End of file with no available line fails.

## `Math`

```oberon
CONST pi = 3.14159265358979; e = 2.71828182845905;
PROCEDURE sqrt(x: REAL): REAL;
PROCEDURE power(base, exp: REAL): REAL;
PROCEDURE exp(x: REAL): REAL;
PROCEDURE ln(x: REAL): REAL;
PROCEDURE log(x, base: REAL): REAL;
PROCEDURE round(x: REAL): REAL;
PROCEDURE sin(x: REAL): REAL;
PROCEDURE cos(x: REAL): REAL;
PROCEDURE tan(x: REAL): REAL;
PROCEDURE arcsin(x: REAL): REAL;
PROCEDURE arccos(x: REAL): REAL;
PROCEDURE arctan(x: REAL): REAL;
PROCEDURE arctan2(y, x: REAL): REAL;
PROCEDURE sinh(x: REAL): REAL;
PROCEDURE cosh(x: REAL): REAL;
PROCEDURE tanh(x: REAL): REAL;
PROCEDURE arcsinh(x: REAL): REAL;
PROCEDURE arccosh(x: REAL): REAL;
PROCEDURE arctanh(x: REAL): REAL;
```

The constants are source constants rounded to binary32 by the lexer. Each function calls the corresponding `float` libm operation. `log(x, base)` is `ln(x) / ln(base)`. `round(x)` is binary32 `floor(x + 0.5)`, so halfway cases round toward positive infinity. Domain errors yield NaNs and overflow yields infinities under the compiler's ordinary IEEE `REAL` rules.

## `Strings`

```oberon
PROCEDURE Length(s: ARRAY OF CHAR): INTEGER;
PROCEDURE Insert(source: ARRAY OF CHAR; pos: INTEGER; VAR dest: ARRAY OF CHAR);
PROCEDURE Append(extra: ARRAY OF CHAR; VAR dest: ARRAY OF CHAR);
PROCEDURE Delete(VAR s: ARRAY OF CHAR; pos, n: INTEGER);
PROCEDURE Replace(source: ARRAY OF CHAR; pos: INTEGER; VAR dest: ARRAY OF CHAR);
PROCEDURE Extract(source: ARRAY OF CHAR; pos, n: INTEGER; VAR dest: ARRAY OF CHAR);
PROCEDURE Pos(pattern, s: ARRAY OF CHAR; pos: INTEGER): INTEGER;
PROCEDURE Cap(VAR s: ARRAY OF CHAR);
```

All scans are bounded by the actual array length. `Length` returns the index of the first null, or the full bound when no null exists. `Pos` treats an unterminated input's full bound as content, starts at `pos`, returns `pos` for an empty pattern, and returns -1 when no match exists.

`Insert`, `Append`, and `Replace` truncate to leave a null in every nonempty destination. `Delete` removes at most the available suffix and moves the terminator. `Extract` truncates to the requested count, the available suffix, and destination capacity, and terminates every nonempty destination. `Cap` changes only ASCII `a` through `z`. The loop directions define self-insertion, self-append, self-replacement, and in-place extraction as though the source had been copied first.

The mutators assert every required terminator before their first write. Positions must lie from zero through the current string length, counts must be nonnegative, and a destination that receives a string must have positive length. A violated precondition is the language runtime failure `assertion failed`.

## `Files`

```oberon
TYPE File = POINTER TO Handle;
TYPE Rider = RECORD eof: BOOLEAN; res: INTEGER END;
PROCEDURE Old(name: ARRAY OF CHAR): File;
PROCEDURE New(name: ARRAY OF CHAR): File;
PROCEDURE Register(f: File); PROCEDURE Close(f: File); PROCEDURE Purge(f: File);
PROCEDURE Delete(name: ARRAY OF CHAR; VAR res: INTEGER);
PROCEDURE Rename(old, new: ARRAY OF CHAR; VAR res: INTEGER);
PROCEDURE Length(f: File): INTEGER; PROCEDURE GetDate(f: File; VAR t, d: INTEGER);
PROCEDURE Set(VAR r: Rider; f: File; pos: INTEGER);
PROCEDURE Pos(VAR r: Rider): INTEGER; PROCEDURE Base(VAR r: Rider): File;
PROCEDURE Read(VAR r: Rider; VAR x: BYTE); PROCEDURE Write(VAR r: Rider; x: BYTE);
PROCEDURE ReadInt(VAR r: Rider; VAR i: INTEGER); PROCEDURE WriteInt(VAR r: Rider; i: INTEGER);
PROCEDURE ReadReal(VAR r: Rider; VAR x: REAL); PROCEDURE WriteReal(VAR r: Rider; x: REAL);
PROCEDURE ReadNum(VAR r: Rider; VAR i: INTEGER); PROCEDURE WriteNum(VAR r: Rider; i: INTEGER);
PROCEDURE ReadString(VAR r: Rider; VAR s: ARRAY OF CHAR); PROCEDURE WriteString(VAR r: Rider; s: ARRAY OF CHAR);
PROCEDURE ReadSet(VAR r: Rider; VAR s: SET); PROCEDURE WriteSet(VAR r: Rider; s: SET);
PROCEDURE ReadBool(VAR r: Rider; VAR b: BOOLEAN); PROCEDURE WriteBool(VAR r: Rider; b: BOOLEAN);
PROCEDURE ReadBytes(VAR r: Rider; VAR buf: ARRAY OF BYTE; n: INTEGER);
PROCEDURE WriteBytes(VAR r: Rider; VAR buf: ARRAY OF BYTE; n: INTEGER);
PROCEDURE GetError(VAR msg: ARRAY OF CHAR);
```

`Handle` and the fields that bind a rider to a file and position are private. `Rider.eof` and `Rider.res` are public. `Old` opens an existing regular file for update when possible and read-only otherwise. `New` creates an unnamed temporary file and remembers its requested path. `Register` writes that temporary content to the path and replaces an existing entry. `Close` flushes while leaving the handle usable. `Purge` truncates the content. `Delete` removes a directory entry without invalidating open handles, and `Rename` renames an entry.

`Set` accepts positions from zero through `Length(f)`, binds the rider, clears `eof`, clears `res`, and stores the position. `Pos`, `Base`, and every transfer assert that the rider is bound. Every procedure that dereferences a `File` asserts that it is not `NIL`.

Each transfer begins at the rider's stored position and advances it by the bytes actually transferred. A complete read clears `eof`; a read stopped by end of file sets it. Writes do not change `eof`. Complete transfers set `res` to zero. Fixed-width and bulk operations put the missing byte count in `res`; malformed encoded values use one. Scalar read destinations change only after the complete encoding arrives. `ReadBytes` may change the prefix received and transfers at most the smaller of `n` and the actual buffer length. `WriteBytes` requires `0 <= n <= LEN(buf)`.

`INTEGER`, `REAL`, and `SET` use their complete four-byte target representations: little-endian signed 32-bit, little-endian IEEE binary32, and little-endian 32-bit bit set. Booleans use one byte, zero or one. Compact integers use signed base-128 with continuation bytes carrying seven low bits and the terminating byte carrying six value bits and one sign bit. Strings include their terminating null byte. An undersized `ReadString` destination becomes empty after the stored value is consumed; a missing stored terminator consumes through end of file and also returns an empty destination.

`GetDate` encodes `hour * 4096 + minute * 64 + second` in `t` and `year * 512 + month * 32 + day` in `d`, using local time. An operating-system failure leaves both destinations unchanged.

Expected operating-system failures are results rather than runtime traps. `Old` and `New` return `NIL`; `Delete` and `Rename` return nonzero; failed positioning and transfers preserve scalar destinations and record their partial progress. The implementation retains a bounded message for the most recent failed or malformed file operation. Success does not clear it. `GetError` copies a terminated, possibly truncated message into a nonempty destination and leaves a zero-length destination untouched. File operations do not print diagnostics.

## Target dependence

The source interfaces are portable, but raw file encodings and exact libm results are pinned to the supported `amd64_sysv` target. That target has little-endian 32-bit `INTEGER`, `SET`, and IEEE binary32 `REAL`. The exact last bit of a math result and the C spelling of non-finite output are target-library properties.
