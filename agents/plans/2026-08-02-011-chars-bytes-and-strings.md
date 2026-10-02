# Slice: CHAR, BYTE, strings, and character arrays

## Context

After Slice 10 the compiler supports INTEGER, BOOLEAN, SET, and REAL, type declarations, named and inline fixed arrays, multidimensional arrays, checked indexing, whole-array assignment, and `LEN`. Every scalar the compiler knows is four bytes wide and travels in a QBE word or single. Array layout already asks the element type for its size and alignment rather than assuming four, but no type has ever answered anything else.

The lexer already recognizes both string forms of Report 3. It produces a token for a quoted string and a separate token for the `41X` ordinal form, and nothing downstream consumes either one. Semantic analysis has no CHAR type, no BYTE type, and no notion of a constant that is not a single scalar value. The IR has no way to emit a data object with contents, so no constant bytes can reach the generated program.

This slice adds the `CHAR` and `BYTE` basic types, string literals and string constants, the compatibility rules that connect strings with CHAR and with character arrays, character array comparison, CHAR case labels, `ORD` for CHAR, `CHR`, and character output. It is the first slice in which a value occupies fewer than four bytes and the first in which the compiler emits initialized data.

Report sections implemented: section 3 for both string forms; section 6.1 for `CHAR` and `BYTE` and for the compatibility of `BYTE` with `INTEGER`; section 8.2.4 for the relations on CHAR and on character arrays; section 9.1 exception 2 for assigning a string to a character array and a single-character string to a CHAR; section 9.5 for CHAR case labels and label ranges; section 10.1 for CHAR and BYTE parameters and results; section 10.2 for `ORD` of a CHAR and for `CHR`; and section 11 for exported CHAR, BYTE, and string declarations.

Implementation starts from a reviewed Slice 10 with a green common gate. Slice 10's shared array descriptors, its per-type layout rule, its checked index instruction, and its byte-copy instruction are the baseline that this slice builds on rather than code to rework around.

## Language and representation decisions

### CHAR is one byte and a string literal is a sequence of source bytes

A `CHAR` value is one unsigned byte, so the character set has exactly the 256 ordinals 0 through 255. `ORD` of a CHAR is that ordinal and `CHR` maps an ordinal back. The ordering relations compare ordinals.

The compiler assigns no further meaning to the upper half of that range. A quoted string literal denotes the bytes of its source text exactly as they appear in the file, so an ASCII source file produces one CHAR per character and a UTF-8 source file produces one CHAR per byte of a multi-byte character. Nothing in the compiler decodes, re-encodes, or validates those bytes, and `Out.Char` writes the byte it is given. This is the one choice that needs no character set table and no encoding assumption, and it makes the bytes a program writes identical to the bytes its source file contained.

The `nnX` form is a string of one character, not a separate kind of literal. Report 3 puts it in the `string` production and calls it "a single-character string specified by the ordinal number of the character", so the parser turns it into the same expression node as `"A"`. This matters in one visible place: `41X` may be assigned to a character array of length two or more, because a string may. The two references disagree here. Project Oberon's scanner returns a CHAR symbol for the `nnX` form, so `41X` is a character literal there and cannot be assigned to an array. OBNC returns a string, which is what the Report describes, but it stores a string constant as a C string and so turns `0X` into a string of no characters. This compiler keeps a string's bytes in a length-carrying buffer, so `0X` is an ordinary one-character string whose character is the null character and `ch := 0X` is the way to write that character.

Declined: restricting a quoted literal to ASCII and requiring `nnX` for everything above 127. It would forbid ordinary text in a literal without making any behaviour better defined.

Declined: reading a literal's characters as Latin-1 code points, so that one non-ASCII source character becomes one CHAR. The source file is UTF-8 because that is what the compiler already reads, and reinterpreting its bytes under a second encoding would make the emitted bytes differ from the source bytes for no stated benefit.

### A string is a constant and never a variable's type

There is no string variable, no string parameter, and no string field. A string appears only as a literal or as a constant declared from one, so its type belongs to the constant machinery and not to the storage machinery. The semantic type gains a string case carrying the number of characters, and the constant value gains a string case carrying the bytes. No variable declaration, formal type, or array element type may name it, and none can, because the source has no identifier for it.

A constant declaration keeps the string type of its right-hand side. `CONST Prompt = "> "` declares a string constant of two characters, and `CONST Quote = 22X` declares a string constant of one character. Project Oberon converts a one-character string to a CHAR constant at the declaration; that is declined here, because it would make a named constant behave differently from the literal it was declared from. Applying the single-character rule at each use site instead is a rule the compiler needs anyway for literals.

The number of characters is the count the source wrote. It does not include a terminator. A terminator exists only where the Report puts one, which is in a character array that a string was assigned to.

### A single-character string is an ordinary CHAR value

Report 9.1 says a single-character string may be assigned to a CHAR variable, and Report 8.2.4 lets CHAR values be compared. The Report's own examples compare `ch` with `"A"` and with `22X`. Rather than write that exception into each context separately, a string of exactly one character lowers to a CHAR value wherever an expression is lowered. The comparison `ch >= "A"`, the argument `p("A")`, the assignment `ch := 0X`, and the result `RETURN "A"` all follow from that one rule with no further cases.

The empty string is not a single-character string, so it has no CHAR value. Project Oberon agrees, and OBNC does not: OBNC treats any string of length one or less as convertible to CHAR. The Report says "single-character strings", and `""` has no characters, so this compiler follows Project Oberon.

A string of two or more characters has no scalar value at all. Lowering one in an ordinary expression is a source diagnostic. The four contexts that accept a longer string examine the expression before lowering it: assignment to a character array, a relation whose other operand is a character array or another string, a case label, and a constant declaration.

### Strings assign to character arrays by copying the characters and one null

Report 9.1: a string may be assigned to any array of characters provided the number of characters in the string is less than the length of the array, and a null character is appended. A character array here is a one-dimensional array whose element type is CHAR.

The compiler checks the length rule when it sees the assignment, so a string that does not leave room for the terminator is a source diagnostic and never a runtime failure. The copy moves exactly the characters and one null character, which is the string's character count plus one byte. It leaves the rest of the destination untouched.

Declined: zero-filling the remainder of the destination. The Report requires an appended null and nothing more, a program that wants a cleared array can clear it, and filling the rest would make an assignment cost time proportional to the declared length rather than to the value assigned.

Declined: Project Oberon's word-at-a-time copy of the padded literal, which moves whole four-byte groups and can write up to three bytes past the terminator. It would write into elements the assignment does not mention.

An empty string may be assigned to any character array of length one or more, and copies one byte. A string assigned to an array of a non-character element type is a diagnostic, as is a string assigned to a nested array.

### Character array comparison is bounded by both declared lengths

Report 8.2.4 extends all six relations to character arrays. Two operands may be compared when each of them is a character array or a string, and at least one of them is a character array. Two strings are compared by constant folding instead, and a CHAR against a single-character string is an ordinary CHAR comparison, so neither reaches this path.

The comparison walks both operands in step and stops at the first position where the bytes differ, at a null character present in both, or at the shorter operand's length, whichever comes first. The result is the unsigned ordering of the first differing pair, or equality if the walk ran out. The length that bounds a character array is its declared length, and the length that bounds a string is its character count plus one for the terminator that its data object carries.

This is OBNC's rule, which lowers to `strncmp` over the smaller of the two lengths. It gives the expected answer when a character array holds a properly terminated value, it also gives an answer when the array is filled to its last element and holds no null character, and it never reads past a declared length. Project Oberon's comparison instead walks until it finds a null, which reads past the end of an unterminated array; that is declined.

The bound makes a shorter value compare as less than a longer one that starts with it, because the shorter one's terminator is compared against the longer one's next character and a null is below every other ordinal. Constant folding of two strings applies the same rule to the same byte sequences, each with its terminator appended, so a folded comparison and a computed one always agree.

### BYTE is an integer type whose storage is one byte and whose stores are checked

Report 6.1 says the values of `BYTE` are the integers between 0 and 255, and that BYTE is compatible with INTEGER and INTEGER with BYTE. This compiler reads that as one rule in each direction.

Reading a BYTE yields an INTEGER. A designator whose declared type is BYTE produces an INTEGER value when it is loaded, and a function whose result type is BYTE produces an INTEGER value when it is called. Everything that follows is ordinary INTEGER behaviour: arithmetic on two BYTE variables is INTEGER arithmetic, a BYTE variable may index an array, and a BYTE value may be compared with an INTEGER one. BYTE remains a distinct type for declarations, layout, parameter identity, and diagnostics, and it is only the loaded value that is an INTEGER.

Writing a BYTE checks the range. An INTEGER value stored into a BYTE variable, element, parameter, or result must lie from 0 through 255 inclusive. A value the compiler can fold is a source diagnostic and emits no runtime check. Any other value is compared against both bounds before the store, and a failure prints one line and exits. This follows the same shape as the SET element check and the shift count check: a constant is a compile-time error, a dynamic value is a runtime check, and target truncation never gets to define the language.

Both reference compilers truncate instead. Project Oberon gives BYTE the same internal form as INTEGER with a size of one, and its store instruction narrows silently; OBNC generates a C `unsigned char` and lets C narrow. That is declined. The Report defines BYTE's value set as the integers 0 through 255, so a program that stores 256 into a BYTE has no meaning under the Report, and this compiler says so rather than storing 0.

One exception to the reading rule is stated in the Report itself. Report 9.8 says the types of a `FOR` statement's control variable, start, and limit must be INTEGER, so a BYTE control variable is a source diagnostic. The rule is worth keeping for a second reason: the statement's final increment runs past the limit, so a loop ending at 255 would trap on an increment the source never wrote.

`INC` and `DEC` accept a BYTE variable, because their argument is an integer and BYTE is compatible with INTEGER. Both use the same checked store, so incrementing past 255 fails at run time. That is a genuine out-of-range write and not an artefact of the lowering.

A variable parameter still requires an identical type, as Report 10.1 demands. A BYTE variable parameter takes a BYTE actual and nothing else, and an INTEGER variable parameter does not accept a BYTE actual.

There is no BYTE constant and no way to write one. An integer constant is an INTEGER, and it becomes a BYTE only by being stored into one.

### CHR has the same domain as BYTE

`CHR(x)` yields the character with ordinal `x`, and this implementation's characters are the ordinals 0 through 255. An argument outside that range has no result, so a constant argument outside it is a source diagnostic and a dynamic one is checked before the value is used. The check and the message are separate from the BYTE one, matching how each existing dynamic check has its own line.

`CHR` emits no conversion instruction. A CHAR already travels in a word holding its ordinal, so the checked value is the result.

`ORD` gains its CHAR form and emits nothing at all, as its BOOLEAN and SET forms already do.

### CASE selects on CHAR

Report 9.5 says that if the case expression is of type INTEGER or CHAR, all labels must be integers or single-character strings respectively. The selector may now be a CHAR, in which case every label and every range endpoint must be a single-character string or a CHAR constant. An INTEGER selector keeps its existing rule, and a label of the wrong kind for the selector is a source diagnostic.

Labels reduce to ordinals, so the existing machinery for reversed ranges, for overlap across alternatives, and for the trap when no label matches is reused unchanged. The failure message for a CHAR case with no matching label is the existing one.

### Character output

The temporary native `Out` interface gains `Out.Char`, which writes one character. Its runtime implementation writes the byte and nothing else, so a corpus program's standard output is exactly the bytes it asked for.

`Out.String` is not added. Its parameter is an open array of CHAR, and open array formals arrive in Slice 14, so declaring one now would mean inventing a parameter form the language does not yet have and then removing it. A corpus program prints a character array by looping until it reaches the null character, which is the idiomatic Oberon and which additionally proves where the terminator sits. When Slice 17 replaces this interface with `lib/Out.Mod`, `Out.String` is a few lines of Oberon over `Out.Char`.

Declined: a hidden parameter form used only by the fallback `Out` interface so that `Out.String` could exist one slice early.

## Semantic types and module interfaces

### The semantic type and the constant value both gain cases

The semantic type gains `CHAR`, `BYTE`, and the string case carrying a character count. Its layout helper answers one byte with alignment one for CHAR and for BYTE. Its scalar helper answers the byte-wide IR type for both, and answers nothing for the string case, so no string can reach a load, a store, an argument, or a result by accident.

The constant value gains a CHAR case carrying an ordinal and a string case carrying shared owned bytes. The string case makes the constant value no longer copyable, in the same way that Slice 10's array descriptors made the semantic type no longer copyable. Audit every place a constant value is copied today and clone deliberately. Sharing the bytes keeps a clone cheap and keeps one buffer behind a constant that several modules import.

Diagnostics print `CHAR` and `BYTE` for the basic types and "string of N characters" for the string case, so an existing message such as "argument 1 has type X, expected Y" reads correctly with no change to its shape.

### One function decides assignment compatibility

Assignment compatibility is currently the equality of two types, tested in three places. It now has five outcomes, so it becomes one function that names them: an ordinary scalar store, a checked store into a BYTE, a CHAR taken from a single-character string, a string copied into a character array, and Slice 10's whole-array copy of identical types. Anything else is a diagnostic.

The assignment statement, the value parameter, and the `RETURN` expression all ask that one function and act on its answer. A variable parameter does not: it requires identical types and keeps its own rule. The same function backs the predefined operations' argument checks, so `ORD("A")` and a CHAR argument to a user procedure agree with assignment.

### Interfaces carry the new members without new machinery

An exported CHAR constant, an exported string constant, an exported CHAR or BYTE variable, and a procedure with CHAR or BYTE parameters or result all travel through the existing interface members. The only change is that cloning an interface now clones the shared bytes behind a string constant.

A string constant that an imported module declares reaches a client as its bytes. When the client uses it in a comparison, the client emits its own data object from those bytes. No module refers to another module's literal data, and no new symbol crosses a module boundary.

## IR and QBE lowering

### One byte-wide IR type

The IR gains a single byte-wide type used by both CHAR and BYTE. Both are one unsigned byte in storage and both travel in a QBE word, so two IR names would emit identical code. The IR describes machine types rather than source types, and CHAR and BYTE differ only in the source rules that decide what may be stored.

The immediate for that type is one byte. Storage size is one and alignment is one. In every calling position the type is a QBE word, so a CHAR parameter, a BYTE parameter, and either as a result use the word class, and a runtime function that takes one declares a 32-bit integer parameter in C.

Loading uses `loadub` and storing uses `storeb`. Every byte-wide value therefore reaches a register zero-extended and holds 0 through 255, which is why the existing signed word comparisons give the correct unsigned ordering for CHAR and need no new instructions.

This is the first type whose alignment is not four. Module data objects take their alignment from the type, so a character array global is emitted with alignment one and its exact byte size. Procedure slots keep `alloc4`, because that is QBE's smallest allocation and over-alignment is harmless. Slice 10's per-procedure and per-module storage accounting already uses the type's size and alignment and needs no rule change, but it now sees values other than four for the first time and must be exercised with them.

An array of CHAR is exactly its length in bytes with no padding between elements and none at the end, because Slice 10 lays an array out as length times element size.

### String literals are data objects

The IR module gains a list of literals, each a symbol and its bytes. A literal is emitted as a QBE data object with alignment one, holding the string's characters followed by one null character. Every byte is written as a decimal item rather than inside a quoted QBE string, so no escaping rule is needed and every byte value including zero is emitted the same way.

A literal is created only when a string's bytes have to exist at run time, which is a string assigned to a character array and a string compared against a character array. A string that only ever folds emits nothing. Literals are numbered within their module and are not shared between two occurrences of the same text; there is no optimization pass and a second data object costs a few bytes.

The driver's aggregate static-data check from Slice 10 counts literal data along with globals.

### Comparison and copy are runtime calls

Copying a string into a character array reuses Slice 10's byte-copy instruction with a byte count of the string's character count plus one, and its source is the literal's address. No new instruction is needed.

Comparison needs no new instruction either. Semantic analysis emits an ordinary call to `oberon_str_cmp` with the two addresses and the two bounding lengths, and compares the returned integer against zero with the existing word comparison. One runtime helper therefore serves all six relations. Both addresses are ordinary IR addresses, so a global array, a local array, a selected row of a nested array, and a literal all reach it the same way.

Both operands are resolved left to right and each exactly once, so an index expression in either operand runs once and is bounds-checked before the comparison.

## Procedures and predefined operations

CHAR and BYTE work in every role the compiler has: constants for CHAR, module variables, procedure locals, value parameters, variable parameters, and function results. A character array works in the roles Slice 10 gave arrays, which are module variables and procedure locals; a character array parameter waits for Slice 12 along with every other structured parameter.

A single-character string is accepted wherever a CHAR value is, including as a value parameter and as a `RETURN` expression. A longer string is accepted as an actual parameter nowhere in this slice, because the only formal that could take one is an open array and that is Slice 14.

`ORD` accepts CHAR in addition to BOOLEAN and SET. `CHR` is installed in the universe scope with its domain check. Both fold when their argument folds, and the folded and the runtime forms produce identical results.

`LEN` of a character array is its declared length, which counts the element that holds the terminator. `LEN` of a string is a diagnostic: Report 10.2 gives `LEN` a variable, and a string is a constant.

## What remains unsupported after this slice

- Records, record fields, and record layout remain unsupported until Slice 12.
- Fixed-array and record value and variable parameters remain unsupported until Slice 12, so a character array cannot yet be passed to a procedure.
- Pointers, `NIL`, `NEW`, and dereference selectors remain unsupported until Slice 13.
- Open array formals remain unsupported until Slice 14. A string therefore cannot be passed to a procedure unless it has exactly one character, and `Out.String` does not exist yet.
- Open array assignment and its dynamic length check, including the oversized-string case, remain in Slice 14.
- Record extension, type tests, type guards, and the record form of `CASE` remain in Slice 15.
- Procedure types remain in Slice 16.
- There is no string or character library. Slice 17 brings `Strings` and the rest of the portable profile.
- The optional `SYSTEM` module remains outside the core roadmap.

The scalar arithmetic, REAL representation, SET domain, array layout, checked indexing, module initialization, and scalar calling convention from prior slices remain unchanged.

## Changes by file

### src/lexer.rs

Replace the two string tokens with one carrying bytes. A quoted literal yields the bytes of its source text, and the `nnX` form yields one byte, so both reach the parser as the same kind of token. Keep the existing diagnostics for an unterminated string and for an ordinal above 255.

Update the lexer's own tests for the merged token and add cases for the empty string, for `0X` as a one-character string, for a byte above 127 written as `nnX`, and for a quoted literal containing a multi-byte character.

### src/ast.rs

Add a string expression carrying its bytes and its position. Remove the character literal if one exists as a distinct node. Nothing else in the AST changes: CHAR and BYTE are ordinary type names and reach semantic analysis through the existing qualified type form.

### src/parser.rs

Accept a string as a factor and as a case label. Add parser tests for a string factor, a string case label, a label range between two single-character strings, and a string in a constant declaration.

### src/sema.rs

Add CHAR, BYTE, and the string type case, with their layout, their scalar IR type, and their display text. Add the CHAR and string constant values and audit every constant value copy for the loss of copyability.

Install `CHR` in the universe scope and add the CHAR form of `ORD`. Give `CHR` its constant and runtime domain checks.

Add the byte-wide load and the checked byte-wide store. Route every BYTE write through the checked store: assignment, value parameter, function result, `INC`, and `DEC`. Widen a loaded BYTE to INTEGER at the point it is produced, and reject a BYTE control variable in a `FOR` statement with a diagnostic that cites the Report's requirement.

Replace the three equality tests that decide assignment compatibility with the one function described above, and use it for the assignment statement, value parameters, `RETURN`, and predefined argument checks.

Lower a single-character string to a CHAR value in `lower_expr`, and diagnose a longer string reaching an ordinary value context. Add the string assignment path with its compile-time length rule and its byte copy. Add the character array comparison path with its runtime call. Add constant folding for CHAR relations, for string relations under the same bounded rule the runtime uses, and for `ORD` and `CHR`.

Extend the case statement to a CHAR selector, with labels and range endpoints that must be single-character strings or CHAR constants.

Add `Out.Char` to the temporary native interface.

### src/ir.rs

Add the byte-wide type and its immediate. Add the module's literal list. Nothing else: the string copy reuses the byte-copy instruction and the comparison reuses the ordinary call.

### src/qbe.rs

Emit the byte-wide type as a word in every calling position, as `loadub` and `storeb` for memory traffic, and as size one with alignment one for storage. Select the load and store mnemonics from the type rather than from its register class, which they are derived from today.

Emit each literal as a data object with alignment one, its bytes as decimal items, and a trailing zero byte. Emit a data object's alignment from its type instead of from its size.

### runtime/oberon.c

Add:

- `oberon_out_char(int32_t c)`, which writes the low byte.
- `oberon_str_cmp(const unsigned char *a, int32_t alen, const unsigned char *b, int32_t blen)`, which compares at most the smaller of the two lengths and returns a negative number, zero, or a positive number. It delegates to `strncmp`, which stops at the first difference and at a shared null character and which compares as unsigned characters.
- `oberon_byte_range(void)`, which prints `BYTE value out of range` and exits.
- `oberon_chr_range(void)`, which prints `CHR argument is outside CHAR range` and exits.

### agents/architecture.md

Add a section covering the character set, the one-byte representation of CHAR and BYTE, the word class those types use in calls, the string literal's data object, the terminator rule for a string assignment, the bounded comparison rule, the BYTE range check, and the `CHR` domain. Note in the existing scalar representation section that not every scalar is four bytes any more.

## New corpus modules

`tests/corpus/` compiles each module, runs it, and compares standard output.

- `Chars.Mod` covers CHAR module variables, locals, constants, value parameters, variable parameters, and a CHAR function result. It uses quoted and ordinal literals, both boundary ordinals, `ORD` and `CHR` in both directions, and all six relations. Every relation is computed twice, once folded into a constant declaration and once at run time from variables, and the two columns must match, following `Sets.Mod`.
- `Strings.Mod` assigns strings to character arrays of several lengths, including the exact fit where the string's characters and its terminator fill the array, and including the empty string. It proves where the terminator sits by scanning for it, prints each array by looping until the null character, and assigns single-character quoted and ordinal literals both to a CHAR variable and to a two-element array. It also shows that a string assignment is a copy and not an alias, by mutating the destination and printing an untouched second array assigned from the same literal.
- `StringCompare.Mod` covers equal values, a proper prefix, a difference in the first position, a difference in the last, and a character array filled to its last element with no null character. It compares an array against a literal, an array against another array, and two literals, and it prints a folded and a computed result side by side for each pair.
- `CharCase.Mod` runs a `CASE` on a CHAR with single labels, ranges, a CHAR constant label, and an ordinal literal label, and it reaches every alternative.
- `CharTable.Mod` declares an array of character arrays, assigns a string to each row, indexes rows and characters, compares two rows, and applies `LEN` to the table and to a selected row.
- `Bytes.Mod` covers BYTE module variables, locals, an array of BYTE, both boundary values, assignment in both directions between INTEGER and BYTE, arithmetic mixing the two, `INC` and `DEC` on a BYTE, a BYTE value parameter given an INTEGER actual, a BYTE variable parameter, and a BYTE function result.
- `PredefinedShadow.Mod` is extended so a source declaration can shadow `CHR` while the existing predefined names keep working in their scopes.

`tests/corpus/modules/char-api/` supplies the cross-module coverage.

- `CharTypes.Mod` exports a named character array type, a CHAR constant, a string constant, and CHAR and BYTE variables that its body initializes.
- `CharApi.Mod` imports it, declares a variable of the exported type, assigns the imported string constant to it, compares it against the imported read-only array, uses the imported CHAR constant in a relation and in a case label, reads the imported BYTE variable into an INTEGER, and prints the results.

`tests/errors/` must fail with exact diagnostics.

- `StringAssignBad.Mod` covers a string whose characters exactly fill the destination, a longer string, a string assigned to an array of INTEGER, a string assigned to a nested array, and a multi-character string assigned to a CHAR.
- `StringUseBad.Mod` covers a string in arithmetic, a string compared against an INTEGER, a string as a variable parameter actual, `LEN` of a string, and a multi-character string used as an ordinary value.
- `CharBad.Mod` covers CHAR in arithmetic, a CHAR assigned to an INTEGER without `ORD`, an INTEGER assigned to a CHAR without `CHR`, a CHAR array index, a CHAR compared against a character array, `ORD` and `CHR` with wrong arity and wrong argument types, and `CHR` of a constant outside 0 through 255.
- `ByteBad.Mod` covers a constant outside 0 through 255 assigned to a BYTE, an INTEGER actual for a BYTE variable parameter, a BYTE actual for an INTEGER variable parameter, a BYTE control variable in a `FOR` statement, and a BYTE where a REAL is required.
- `CharCaseBad.Mod` covers an INTEGER label under a CHAR selector, a CHAR label under an INTEGER selector, a multi-character string label, a reversed CHAR range, and two overlapping CHAR ranges.

`tests/errors/modules/char-import-write/` proves that an imported character array, one of its elements, and an imported CHAR variable all reject assignment.

`tests/failures/` compiles each module, runs it, and compares standard error after a nonzero exit.

- `ByteRangeHigh.Mod` stores a dynamic 256 into a BYTE.
- `ByteRangeLow.Mod` stores a dynamic -1 into a BYTE.
- `ByteIncRange.Mod` increments a BYTE holding 255.
- `ChrRangeHigh.Mod` calls `CHR` with a dynamic 256.
- `ChrRangeLow.Mod` calls `CHR` with a dynamic -1.
- `CharCaseNoMatch.Mod` runs a CHAR case whose selector matches no label and expects the existing message.

## Verification

1. Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `git diff --check`.
2. Run every new positive binary directly. Confirm a zero exit status, empty standard error, and byte-for-byte expected standard output.
3. Confirm every folded and computed column in `Chars.Mod` and `StringCompare.Mod` agrees, and that the comparison pairs include one where the two values differ only past the terminator.
4. Inspect `build/Chars.ssa`. Confirm CHAR variables load with `loadub` and store with `storeb`, that CHAR parameters and results use the word class, that `ORD` and `CHR` emit no conversion, and that `CHR` of a dynamic argument checks both bounds before the value is used.
5. Inspect `build/Bytes.ssa`. Confirm every BYTE store is preceded by a range check, that a constant in range emits no check, that a loaded BYTE is used as an ordinary word, and that the array of BYTE reserves exactly its length in bytes.
6. Inspect `build/Strings.ssa`. Confirm each literal is a data object with alignment one whose last item is a zero byte, that a string assignment copies exactly the character count plus one, and that no code touches the remaining elements of the destination.
7. Inspect `build/StringCompare.ssa`. Confirm each comparison passes both addresses and both bounding lengths, that a literal's length includes its terminator and a character array's is its declared length, that each operand is resolved once, and that the six relations differ only in the comparison against the returned value.
8. Inspect globals and slots. Confirm a character array global has alignment one and its exact size, that a CHAR global is one byte, and that the four-byte types are unchanged.
9. Rebuild unchanged INTEGER, SET, REAL, and array programs and compare their IL with the pre-slice results. Adding a byte-wide type and literal data must not alter existing code generation.
10. Compile the cross-module gate and confirm the client emits its own data object for the imported string constant, that no literal symbol crosses a module boundary, and that the imported read-only array and CHAR variable reject assignment.
11. Confirm that no newly valid character, byte, or string program can reach an unsupported diagnostic, a panic, a QBE parse error, an assembler error, or a linker error, and that a character array parameter and a string parameter stop at their stable Slice 12 and Slice 14 diagnostics.

## Order of work

1. Merge the two string tokens into one carrying bytes, add the string expression to the AST, and parse a string as a factor and as a case label.
2. Add CHAR and BYTE as basic types with their one-byte layout, their byte-wide IR type, `loadub` and `storeb` lowering, and their word class in calls. Add `ORD` for CHAR, `CHR` with its domain check, and the checked BYTE store, and route every BYTE write through it.
3. Add the string type and the string constant value, the single-character CHAR rule, the one assignment-compatibility function, and constant folding for CHAR and string relations.
4. Add literal data objects to the IR and to QBE emission, and count them in the driver's static-data check.
5. Add string assignment with its length rule and its byte copy, and character array comparison with its runtime helper.
6. Extend the case statement to a CHAR selector and its labels.
7. Add `Out.Char` and its runtime implementation.
8. Add the positive, negative, runtime-failure, cross-module, and boundary gates, and a regression module for every bug found during implementation.
9. Update the architecture document and complete the verification list.
