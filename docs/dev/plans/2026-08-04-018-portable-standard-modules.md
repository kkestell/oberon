# Slice: Portable standard modules

## Context

The starting point is commit `0fd3906` (`Add procedure types and indirect calls`), the completed roadmap Slice 16 described by [the procedure-types plan](2026-08-04-017-procedure-types-and-indirect-calls.md). The full gate passes: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test`, including 42 unit tests and the two integration tests.

The compiler now implements every grammar production in the May 2016 Report. It discovers source modules in the root directory and then `lib/`, carries exported types and procedures through in-memory interfaces, lowers open arrays and record `VAR` parameters with explicit hidden arguments, and links one QBE unit against BDWGC and the C runtime. There is no `lib/` directory yet. `Out` is still a synthesized native interface with only `Char`, `Int`, and `Ln`, so every existing observable corpus program bypasses source-module compilation for its output calls.

This slice implements roadmap Slice 17. It defines the first portable library profile and supplies `Out`, `In`, `Math`, `Strings`, and `Files` as bundled Oberon modules. The vendored OBNC 0.17.2 definitions and tests are the compatibility reference. The implementation is written independently and does not copy the vendored compiler or library code.

The Report does not define these five standard modules. No new core-language rule is implemented. The slice exercises section 4's export and qualified-name rules, sections 6.2 through 6.4 for open arrays, records, and pointers, section 9.2 for calls, section 10.1 for value, `VAR`, and open-array parameters, and section 11 for imports, read-only exported variables, module initialization, and implementation-defined module lookup. The optional `SYSTEM` module in section 11.1 remains outside the slice.

## Supported profile

The public surface follows the vendored OBNC definitions, including OBNC's useful `Out.Hex`, `In.Line`, and `Files.GetError` extensions. Names, capitalization, parameter modes, result types, exported fields, and the lower-case Math procedure names match those definitions.

- `Out` exports `Open`, `Char`, `String`, `Int`, `Hex`, `Real`, and `Ln`.
- `In` exports the read-only client variable `Done` and the procedures `Open`, `Char`, `Int`, `Real`, `String`, `Name`, and `Line`.
- `Math` exports the constants `pi` and `e` and the procedures `sqrt`, `power`, `exp`, `ln`, `log`, `round`, `sin`, `cos`, `tan`, `arcsin`, `arccos`, `arctan`, `arctan2`, `sinh`, `cosh`, `tanh`, `arcsinh`, `arccosh`, and `arctanh`.
- `Strings` exports `Length`, `Insert`, `Append`, `Delete`, `Replace`, `Extract`, `Pos`, and `Cap`.
- `Files` exports the pointer type `File`, the record type `Rider`, the public fields `Rider.eof` and `Rider.res`, and the procedures `Old`, `New`, `Register`, `Close`, `Purge`, `Delete`, `Rename`, `Length`, `GetDate`, `Set`, `Pos`, `Base`, `Read`, `ReadInt`, `ReadReal`, `ReadNum`, `ReadString`, `ReadSet`, `ReadBool`, `ReadBytes`, `Write`, `WriteInt`, `WriteReal`, `WriteNum`, `WriteString`, `WriteSet`, `WriteBool`, `WriteBytes`, and `GetError`.

`Input`, `Input0`, and `XYplane` are not aliases for `In` and are not part of this profile. Their terminal and graphical behavior is outside the roadmap's portable gate.

## Bundled source and the private runtime boundary

Add `lib/Out.Mod`, `lib/In.Mod`, `lib/Math.Mod`, `lib/Strings.Mod`, and `lib/Files.Mod`. Each is lexed, parsed, analyzed, lowered, initialized, and exported through the same path as a user module. Root-directory lookup continues to win, so a user's `Out.Mod` or `Math.Mod` replaces the bundled module for that build. The driver removes the missing-source `Out` fallback and semantic analysis removes `out_interface`.

The bundled modules import a compiler-private module named `OberonRuntime`. The driver provides its synthesized interface only when the importer is one of the files found in the configured bundled-library directory. A user source module with the same import does not receive the private interface. A user-defined `OberonRuntime.Mod` remains an ordinary source module for user imports, but it cannot replace the private interface used by bundled modules. Dependency resolution recognizes this one bundled import before consulting either the completed-module map or source lookup, so a user module compiled earlier under the same name cannot capture the bundled import. The private interface is returned directly to the importing bundled module and is never entered in the program-wide completed-module map, so compiling a bundled module first cannot make the native interface visible to a later user import.

Change source lookup to return both the path and whether it came from the root or bundled directory. Pass that origin through dependency compilation instead of inferring trust from a string prefix or a canonicalized path. Compiling one of the actual `lib/*.Mod` files as the root recognizes that exact bundled path. A root shadow with the same module name remains user source and receives no private access.

`OberonRuntime` contains only the low-level procedures that source Oberon cannot implement: byte I/O on standard streams, checked token scanners, finite-width formatting, libm calls, and operating-system file operations. Its semantic signatures use only existing basic types, scalar `VAR` parameters, and open arrays. It exports no types, variables, constants, addresses, allocation entry points, or general call-by-symbol operation. This is a closed compiler implementation seam, not a source-language foreign-function interface.

Put the library helpers in `runtime/standard.c` and keep allocation, language checks, and predefined operations in `runtime/oberon.c`. Link both C files on every build. Prefix every new native symbol with `oberon_lib_`. The C declarations follow the existing ABI literally: `INTEGER`, `BOOLEAN`, and `SET` use 32-bit arguments; `REAL` uses C `float`; `CHAR` and `BYTE` use an unsigned byte in memory and a 32-bit calling slot; every scalar `VAR` parameter is an address; and every open array is an address followed by its 32-bit length. Inspect the generated calls rather than creating a parallel ABI description in the backend.

No native helper receives an Oberon record. `Files.Mod` keeps the source-level `File` and `Rider` representations in Oberon and passes integer handles, positions, status values, and byte buffers to the C layer. This keeps record layout, pointer tracing, extension, field visibility, and record-`VAR` descriptors under the ordinary compiler rules.

## `Out`

`Out.Open` is a no-op. `Out.Char` writes exactly one byte. `Out.String` writes bytes from the start of its open character array until the first null character or the supplied length, whichever comes first. A full, unterminated array is therefore bounded and writes all of its bytes without reading past the actual.

`Out.Int(i, n)` writes a decimal INTEGER without a plus sign, left-padded with spaces to a minimum positive width `n`. A zero or negative width adds no padding. The complete 32-bit range, including `MIN(INTEGER)`, is formatted without negating it in Oberon.

`Out.Hex(i)` writes one leading space followed by exactly eight uppercase hexadecimal digits representing the INTEGER's 32-bit pattern. `Out.Real(x, n)` writes binary32 values in uppercase exponential form with six fractional digits, a signed exponent with at least two digits, no leading plus on the mantissa, and left space padding to minimum positive width `n`. Tests pin finite values, negative zero, infinities, and NaNs on the supported C/QBE target. `Out.Ln` writes one line-feed byte; no procedure flushes implicitly except as required by normal C stream behavior at process exit.

The Oberon procedures are the public procedure values. Calls and assignments name `.Out.Char` or the corresponding source symbol, while the wrapper body alone calls an `oberon_lib_` symbol. Existing procedure-value tests must continue to accept `Out.Char` without treating the private runtime procedure as public.

## `In`

`In.Done` starts as `FALSE` and is assigned by every public operation. `In.Open` does not rewind standard input. It sets `Done` to `TRUE` only when no earlier input operation consumed a byte, following the behavior of the vendored OBNC implementation rather than the stronger abort described by its interface comment.

`In.Char` reads the byte at the current position without skipping whitespace. At end of file it sets `Done` to `FALSE` and leaves the destination unchanged. `In.Line` also starts at the current position. It consumes through line feed or end of file, stores the prefix that fits with a null terminator, and reports `Done = FALSE` when truncation occurred or no line was available. An empty line is a successful empty string.

`In.Int`, `In.Real`, `In.String`, and `In.Name` skip preceding ASCII whitespace. `In.Int` accepts an optional minus sign followed by decimal digits, or up to eight uppercase hexadecimal digits followed by `H`; a hexadecimal value is the corresponding 32-bit pattern. It rejects malformed and out-of-range tokens without changing the destination. `In.Real` accepts an optional leading plus or minus followed by the Report's decimal real spelling with an optional signed `E` scale factor, then rounds once to binary32. It rejects malformed input and non-finite conversion results. `In.String` accepts a quoted byte string or a hexadecimal ordinal followed by `X`. `In.Name` reads one non-whitespace graphical byte sequence. These scanners consume the complete token they started even when it is malformed or too large, so the next call begins at a defined boundary.

Every successful character-array read appends a null character. A destination of length zero is left untouched and fails. A destination of length one can receive only the empty string. `In.String` and `In.Name` return an empty terminated string and `Done = FALSE` when a nonempty result does not fit. `In.Line` instead retains the truncated prefix and consumes the rest of the line. The scanner uses explicit byte classifications and does not let the process locale redefine Oberon whitespace, digits, or hexadecimal letters.

## `Math`

`Math.pi` and `Math.e` are source constants whose decimal spellings are rounded by the existing lexer to this compiler's binary32 `REAL`. Every public function accepts and returns `REAL` exactly as the OBNC definition specifies.

The C layer calls the `float` form of each libm operation. `Math.log(x, base)` computes `ln(x) / ln(base)`. `Math.round(x)` computes binary32 `floor(x + 0.5)`, so halfway cases round toward positive infinity. The functions inherit the existing IEEE behavior of `REAL`: a domain error produces a NaN, overflow produces an infinity, and neither becomes a new Oberon trap. Tests observe these cases with relations and exact operations that do not force an invalid `FLOOR` conversion.

No generic intrinsic table or backend math instruction family is added. These are ordinary imported source procedures whose wrappers call typed private runtime procedures, and `-lm` remains the one native dependency.

## `Strings`

Implement `Strings` in Oberon. Every scan is bounded by the actual open-array length. `Length` stops at the first null character and returns the array length when no terminator occurs, so a full array never causes an out-of-bounds read. `Pos` likewise treats the complete bound as content when either read-only input lacks a terminator.

`Insert`, `Append`, and `Replace` truncate to leave a null terminator in a nonempty destination. `Delete` moves the remaining suffix including its terminator. `Extract` truncates to both the requested count and the destination capacity and always terminates a nonempty destination. `Pos` begins at the supplied position, returns that position for an empty pattern, and returns `-1` when no match exists. `Cap` changes only ASCII `a` through `z` and leaves every other byte unchanged.

The mutating procedures assert that every source and destination interpreted as a string has a null terminator within its actual bound. They also assert their documented position and count preconditions, and a destination that must receive a string must have positive length. These assertions run before the first mutation. The implementation chooses loop directions that preserve the original source when `source` and `dest` are the same actual array; self-insertion, self-append, self-replacement, and in-place extraction therefore have defined as-if-copy behavior without a native memory helper.

## `Files`

Define `File* = POINTER TO Handle`, where the private `Handle` record contains one integer native-handle identifier. Define `Rider*` with public `eof` and `res` fields and private `base: File` and `pos: INTEGER` fields. `Old` and `New` ask the runtime for a native handle, allocate the ordinary Oberon wrapper with `NEW`, and return `NIL` on either failure. If wrapper allocation fails after opening a native handle, the wrapper releases that handle before returning. A client can compare, store, and type-test `File` values normally but cannot inspect the private handle.

The C layer owns a process-local table of file objects. Handle zero is invalid. `New` uses an unnamed temporary file and remembers the requested path until `Register`. `Register` writes the temporary contents to that path and replaces an existing directory entry. `Old` opens an existing regular file for update when possible and read-only otherwise. `Close` flushes buffered data but leaves the `File` and its riders usable. `Purge` truncates the underlying content to zero. `Delete` removes a directory entry without invalidating already open handles, and `Rename` renames the directory entry.

Every public procedure that dereferences a `File` first asserts that it is not `NIL`. `Set` also asserts a position from zero through `Length(f)`, then binds the rider, clears `eof` and `res`, and stores the position. `Pos` and `Base` assert that the rider is bound without changing either public status field, as does every read and write. Every transfer starts at the rider's stored position and updates it by the number of bytes actually consumed or produced. A successful read clears `eof`; a read that cannot complete because it reaches the end sets `eof`. Writes do not change `eof`. Every transfer sets `res` to zero on success. A fixed-width or bulk transfer reports the requested byte count that was not transferred. An encoded operation sets `res` to one when its complete bytes arrive but do not form a valid value.

`Read` and `Write` transfer one byte. `ReadInt`, `WriteInt`, `ReadReal`, `WriteReal`, `ReadSet`, and `WriteSet` use the current target's complete in-memory representation: little-endian signed 32-bit INTEGER, little-endian IEEE 754 binary32, and little-endian 32-bit SET. A fixed-width read stages its bytes and changes the destination only after all bytes arrive. A short read advances over the bytes that did arrive, leaves the destination unchanged, sets `eof`, and reports the missing byte count in `res`.

`ReadBool` and `WriteBool` use one byte, with zero for false and one for true. A different input byte is consumed, leaves the destination unchanged, keeps `eof` false, sets `res` to one, and records an invalid-encoding error. `ReadNum` and `WriteNum` use the signed base-128 compact encoding exercised by the vendored OBNC library. `ReadNum` consumes through the first terminating byte or end of file. An overlong or overflowing encoding that reaches a terminating byte leaves the destination unchanged, keeps `eof` false, and sets `res` to one. Reaching end of file before a terminator also leaves the destination unchanged and sets `res` to one, but sets `eof` true.

`ReadString` and `WriteString` transfer the terminating null byte. A `ReadString` destination must have positive length. The read consumes through the stored terminator or end of file and commits the destination only when the complete terminated value fits. An undersized destination becomes an empty string, keeps `eof` false, and sets `res` to one after the complete stored value has been consumed. A missing stored terminator has the same destination and `res` result, sets `eof`, and consumes through end of file. `ReadBytes` transfers at most the smaller of `n` and the actual buffer length and records every unfulfilled requested byte in `res`; `WriteBytes` asserts that zero through `n` bytes exist in the actual buffer. Bulk reads may modify the prefix actually transferred.

All path and string helpers receive an address and an explicit length. They find the null terminator within that bound and never call an unbounded C string operation on source storage. `GetDate` uses local time and the OBNC encoding: hours, minutes, and seconds occupy the fields selected by division by 4096 and 64, while year, month, and day occupy the fields selected by division by 512 and 32.

Expected operating-system failures are library results, not language runtime traps. `Old` and `New` return `NIL`. `Delete` and `Rename` return a nonzero result. A failed `Register` leaves an unregistered temporary file usable, a failed `Close` leaves the handle usable, and a failed `Purge` does not replace the handle. A failed `Length` returns zero. A failed `GetDate` leaves both output variables unchanged. A positioning or transfer failure leaves scalar outputs unchanged, advances a rider only by bytes the operating system reports as transferred, and sets `Rider.res` as described above. A write through a read-only handle follows the same transfer-failure rule.

The C layer retains a bounded message for the most recent failed or malformed file operation. The initial message is empty, and a later success does not erase an earlier failure. `GetError` copies and truncates that message with a terminator when the destination has room, and a zero-length destination is untouched. Library failures do not print unsolicited diagnostics, which preserves the compiler's rule that a successful corpus program writes no standard error.

## Driver and test harness

Extend the corpus harness to use a sibling `.stdin` file as deterministic standard input when one exists. Run every compiled positive and runtime-failure program in its own temporary working directory. The compiler itself still runs from the repository root, so runtime source paths, build outputs, and compile diagnostics remain stable. File tests use relative names inside the temporary directory, remove any registered files they create, and leave the repository untouched.

Build temporary directories with the standard library rather than adding a dependency. Give each directory a name derived from the process and test root, verify that the exact directory was created by the harness, and remove only that exact directory after the child exits. A failed test reports the retained directory before cleanup if preserving artifacts is needed for diagnosis.

The loader tests must cover root shadowing for a standard module, bundled fallback for all five modules, exact case-sensitive spelling, and the private runtime boundary. The private native interface must not appear in generated module initialization, public interfaces, source diagnostics for unrelated imports, or the set of modules a user can import merely because an earlier dependency used it.

## Documentation

Add `docs/standard-library.md` as the public profile reference. It lists every exported declaration, the exact formatting and scanner rules, string truncation and preconditions, file encodings, file error reporting, the supported byte character set, and the implementation-defined dependence on the current `amd64_sysv` little-endian target.

Update [architecture.md](../architecture.md). Remove the native `Out` fallback, describe ordinary bundled-module compilation and root shadowing, document the origin-gated `OberonRuntime` interface, explain the source and C split, add deterministic standard input and temporary working directories to the corpus description, and separate language-runtime checks from fallible library operations.

## Corpus gate

Positive programs compile, link, exit successfully, write no standard error, and compare standard output byte for byte.

- `LibraryOut.Mod` calls every `Out` procedure. It pins zero and negative widths, padding, both INTEGER endpoints, eight-digit hexadecimal patterns, bounded unterminated output, binary32 exponential formatting, non-finite values, and `Out.Char` used through a procedure variable.
- `LibraryIn.Mod` has a sibling `LibraryIn.stdin`. It calls every `In` procedure and observes `Done` after ordinary tokens, signed decimal and hexadecimal endpoints, positive and negative signed reals, real exponents, empty and ordinal strings, whitespace, empty lines, truncation, malformed tokens, zero- and one-byte destinations, and end of file. A second focused module proves `Open` before and after input consumption.
- `LibraryMath.Mod` calls every Math procedure and compares finite results within a binary32 tolerance. It covers both constants, quadrants, negative arguments, halfway rounding on both sides of zero, NaN-producing domains, overflow to infinity, signed zero, and identities that connect the direct, inverse, and hyperbolic forms.
- `LibraryStrings.Mod` calls every Strings procedure. It covers empty strings, bounded `Length` and `Pos` on full unterminated arrays, insertion at both ends, truncation, deletion past the suffix, zero-count deletion and extraction, empty patterns, search from the end, ASCII capitalization, one-byte destinations, and every supported same-array source and destination case.
- `LibraryFiles.Mod` calls every Files procedure in an isolated directory. It covers missing and new files, registration and overwrite, close, purge, delete, rename, length, date-field ranges, rider position and base, end of file, all scalar encodings, compact integers at both INTEGER endpoints, empty and exact-fit strings, partial byte reads, bounded byte writes, a live handle after unlink, a failing operation, and a truncated `GetError` message. Focused companion programs pin short fixed-width values, invalid BOOLEAN and compact-number encodings, undersized and unterminated stored strings, a read-only write, and the state of `pos`, `eof`, `res`, destinations, and the last-error message after each failure.
- `tests/corpus/modules/library-client/` imports a helper that itself imports several bundled modules. Observable initialization and output prove each standard module is compiled once through the normal dependency graph and its exported constants, variable, types, fields, procedures, and procedure values survive another module interface.
- `tests/corpus/modules/library-shadow/` supplies a root `Out.Mod`. Its distinct output proves root source still wins.

Negative modules fail compilation with exact diagnostics.

- Separate missing-module fixtures cover a misspelled standard module, each deliberately excluded module name, and a user import of `OberonRuntime`. Dependency resolution stops at each expected missing import before semantic analysis.
- `LibraryProfileBad.Mod` covers unexported `Files.Handle`, private `Rider` fields, assignment to `In.Done`, wrong modes and element types for open-array procedures, and incompatible procedure values.
- Two private-runtime module graphs exercise opposite dependency orders. One compiles a user-defined `OberonRuntime.Mod` before a bundled module and proves that the user interface cannot capture the bundled import. The other compiles a bundled module before a user import without a source file and proves that the private interface did not leak through the completed-module cache.
- A root-shadow negative graph gives its `Out.Mod` an `OberonRuntime` import and proves that a shadow is user source even though its module name matches a bundled module.
- Existing module privacy, imported-variable mutability, open-array, record-extension, and procedure-signature errors remain unchanged when the relevant types originate in a bundled module.

Runtime-failure modules use existing language checks for invalid source-level preconditions.

- Focused `Strings` failures pin an invalid position, a negative count, a zero-length destination, and an unterminated input passed to a mutator at `assertion failed` without an out-of-bounds access occurring first.
- Focused `Files` failures pin a `NIL` file, an unbound rider, an invalid rider position, a zero-length `ReadString` destination, and a `WriteBytes` count beyond the actual array. Expected operating-system failures remain ordinary library results and are not put in the runtime-failure corpus.

Every public library procedure has at least one behavior assertion and one relevant boundary assertion across these programs. Existing programs that import `Out` keep their byte-for-byte output. Programs that import no standard module retain byte-identical QBE IL.

## Verification

1. Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `git diff --check`.
2. Run every new positive program directly with its recorded standard input and isolated working directory. Confirm exact output, zero status, empty standard error, and no file left outside the test directory.
3. Compare every public declaration in the five `lib/*.Mod` files with the vendored OBNC `.def` files. Confirm exact spelling, modes, open-array ranks, result types, exported fields, and deliberate inclusion of the three documented extensions.
4. Inspect module discovery for each standard module, a repeated transitive import, a root shadow, a direct user import of `OberonRuntime`, and a user-defined module with that name in both dependency orders. Confirm a bundled private import is selected before the completed-module lookup, only an actual bundled importer receives it, and the private interface never enters the shared completed-module map.
5. Inspect generated IL for each private ABI shape: scalar values and results, scalar `VAR` arguments, open CHAR and BYTE arrays, and functions that combine them. Confirm array addresses are followed by lengths, byte values use word call slots, REAL uses the single class, and no record or record descriptor crosses into `runtime/standard.c`.
6. Inspect the emitted module list and `main`. Confirm each used source standard module has one initializer in dependency-first order, `OberonRuntime` has none, an unused standard module emits nothing, and a root shadow prevents its bundled counterpart from being compiled.
7. Inspect procedure values from `Out`, `Math`, and `Files`. Confirm they name the exported source wrappers, remain signature-compatible across interfaces, and never expose a private native symbol.
8. Compare every `Out` byte with the documented format, including both INTEGER endpoints, negative zero, exponent width, non-finite values, and an unterminated full character array. Confirm no helper reads beyond an open-array length.
9. Feed the `In` scanners tokens back to back after successful, malformed, overflowing, and truncated inputs. Confirm each consumes the documented boundary, preserves failed scalar destinations, terminates every writable nonempty character array, and reports `Done` exactly once per public operation.
10. Compare Math results with direct binary32 identities and domain predicates. Confirm wrappers use `float` libm entry points, constants are already binary32, and NaN or infinity does not acquire a library-specific trap.
11. Exercise every Strings procedure with the smallest legal arrays, maximum truncation, and aliased actuals. Exercise full unterminated arrays through `Length` and `Pos`, then pass one to each kind of mutator as a failure case. Confirm all scans remain within their dynamic lengths and every mutator precondition failure reaches `ASSERT` before the first write.
12. Inspect the Files source layouts. Confirm `Handle.id`, `Rider.base`, and `Rider.pos` are private; `Rider.eof` and `Rider.res` retain their documented offsets and visibility; a Rider extension passes through the public wrappers; and no native table address appears in Oberon storage.
13. Inspect each file's bytes after writes. Confirm scalar widths and byte order, null-inclusive strings, compact-number encodings, rider advancement, partial-transfer `res`, and unchanged scalar destinations after every incomplete or malformed read. Reopen registered files and verify the same values through a new `File` wrapper.
14. Force missing-file, failed-delete, failed-rename, and truncated-error-message paths. Confirm they return the documented values, retain a bounded last-error string, emit no standard error, and do not become language runtime failures.
15. Search for `NATIVE_OUT`, `out_interface`, the old three-procedure interface, unbounded C string reads on Oberon arrays, public references to `OberonRuntime`, and unexpected files created by the suite. No correct library client may reach an unsupported diagnostic, panic, QBE error, assembler error, linker error, or out-of-bounds access.

## What remains unsupported

No valid May 2016 core-language program remains deliberately unsupported. Slice 18 still owns the complete conformance matrix, malformed-source closure, combined stress programs, and release gate.

The portable library profile does not include `Input`, `Input0`, `XYplane`, terminal control, graphics, directories, process facilities, networking, locale-dependent text, Unicode character semantics, or the optional `SYSTEM` module. It does not add a public foreign-function interface or let user code import the compiler's private native bindings.

The profile is portable at the Oberon source interface. Raw `Files` scalar encodings are pinned to the only supported target and are not promised to interchange with big-endian or differently sized implementations. Math's correctly rounded last bit and spelling of non-finite `Out.Real` values remain properties of the current binary32 C/libm target and are pinned by this target's gate rather than generalized into a target abstraction.

## Order of work

1. Add origin-aware module lookup and the non-cached, bundled-import-only `OberonRuntime` interface. Remove the native `Out` fallback and pin root shadowing and private-interface isolation before adding library source.
2. Add `runtime/standard.c`, link it, and establish one tested private call for each ABI shape. Keep the native surface limited to the operations the source modules have already demonstrated they need.
3. Implement `Out.Mod` and migrate the existing corpus to the source wrapper without changing output. Complete its formatting and procedure-value tests.
4. Extend the corpus harness for sibling standard input and isolated runtime directories, then implement and test `In.Mod`.
5. Implement `Math.Mod` over typed libm wrappers and pin binary32 finite, domain, infinity, NaN, and rounding behavior.
6. Implement `Strings.Mod` entirely in Oberon. Verify dynamic bounds, termination, truncation, precondition order, and same-array behavior before using it from other library tests.
7. Implement the Files integer-handle runtime and the source `File` and `Rider` wrappers. Add operations in groups: lifecycle and paths, rider positioning, scalar transfers, compact numbers and strings, then bulk bytes and error reporting.
8. Add the transitive-client, root-shadow, privacy, runtime-failure, and every-public-procedure gates. Add a regression module for every bug found during implementation.
9. Add the public library reference, update architecture documentation, run the full verification list, inspect the complete diff, and report the slice ready for independent review.
