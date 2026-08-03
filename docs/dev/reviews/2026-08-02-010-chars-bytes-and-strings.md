# Review: CHAR, BYTE, strings, and character arrays

Reviewed against the uncommitted working tree on top of commit `6f0ed0f` (`fixed arrays and checked indexing`). The implementation follows [the Slice 11 plan](../plans/2026-08-02-011-chars-bytes-and-strings.md).

The untracked Slice 12 plan is outside this review. It contains no implementation changes that affect the Slice 11 result.

## Verdict

The implementation covers the slice's CHAR, BYTE, string, character-array assignment, character-array comparison, CHAR case, predefined-operation, module-interface, IR, QBE, and runtime requirements. One medium-priority finding was confirmed and fixed. No findings remain.

## Finding

### Constant-context procedure-call checks bypass assignment compatibility

`Analyzer::lower_call` uses `assign_kind` for value arguments and converts a BYTE function result into an INTEGER expression value. `Analyzer::check_const_call`, which type-checks a call before rejecting it from a required constant context, instead requires exact argument types and returns the procedure's declared BYTE result type unchanged.

That disagreement adds false diagnostics to otherwise well-typed calls. A single-character string is a valid CHAR value argument, and an INTEGER is a valid BYTE value argument subject to the BYTE range rule. Reading a BYTE function result yields INTEGER, so it is valid in surrounding INTEGER arithmetic. The call still has to be rejected because a user procedure call is not a constant expression, but the compiler must not also claim that these valid conversions are type errors.

The following modules confirmed the problem:

```oberon
MODULE CharByteConstCall;
  IMPORT Support;
  CONST
    A = Support.TakeChar("A");
    B = Support.TakeByte(255);
    C = Support.ReturnByte() + 1;
END CharByteConstCall.
```

`Support.TakeChar` takes a CHAR value, `Support.TakeByte` takes a BYTE value, and `Support.ReturnByte` returns BYTE. Compiling the root module exited unsuccessfully and wrote five diagnostics:

```text
tests/errors/modules/char-byte-const-call/CharByteConstCall.Mod:4:26: argument 1 has type string of 1 character, expected CHAR
tests/errors/modules/char-byte-const-call/CharByteConstCall.Mod:4:9: constant expression contains a procedure call
tests/errors/modules/char-byte-const-call/CharByteConstCall.Mod:5:26: argument 1 has type INTEGER, expected BYTE
tests/errors/modules/char-byte-const-call/CharByteConstCall.Mod:5:9: constant expression contains a procedure call
tests/errors/modules/char-byte-const-call/CharByteConstCall.Mod:6:30: operator '+' requires two INTEGER, two REAL, or two SET operands, found BYTE and INTEGER
Error: 5 error(s)
```

Only the three procedure-call diagnostics are correct.

### Resolution

`Analyzer::check_const_call` now uses `assign_kind` for value parameters while retaining exact type identity for variable parameters. It also maps a declared BYTE result to the INTEGER type produced when the call is read. The `char-byte-const-call` regression now reports one `constant expression contains a procedure call` diagnostic for each declaration and no false argument or arithmetic diagnostic.

## Verified implementation

The lexer produces one byte-carrying token for quoted strings and ordinal strings. `0X` remains a one-character string containing the null byte, `0FFX` carries byte 255, and a quoted UTF-8 character retains all of its source bytes. The parser accepts strings as factors, constants, case labels, and label endpoints.

CHAR and BYTE occupy one byte in globals, arrays, and local slots. Loads use `loadub`, stores use `storeb`, and parameters and results use the QBE word class. The unchanged `Params.ssa` still has SHA-256 `bc6627e4199e252930401829769ea56477ae76987738f442f8111d8121233291`, matching the pre-slice value.

CHAR remains distinct from INTEGER and BYTE in semantic checking. Single-character strings become CHAR values at scalar use sites. `ORD` and `CHR` emit no conversion instruction. Dynamic CHR arguments check both bounds before use, and constant out-of-range arguments are source diagnostics.

BYTE loads become INTEGER expression values. Every source-level write checks the 0 through 255 domain when the value is not already known to be valid. This includes assignments, value arguments, function results, `INC`, and `DEC`. A targeted procedure program returned a single-character string from a CHAR function, passed a dynamic INTEGER value of 255 to a BYTE value parameter, and returned that value from a BYTE function. It exited successfully and printed `Q255 255`. Changing the BYTE result to 256 and changing the BYTE argument to -1 each produced `BYTE value out of range` and a nonzero exit.

String assignment emits one byte-aligned literal data object and copies exactly the character count plus one. The positive string program demonstrates exact fit, early termination, an empty string, ordinal strings, preservation of bytes beyond the terminator, and independent destination storage.

Character-array comparisons pass both operand addresses and both bounds to `oberon_str_cmp`. String bounds include the terminator, and array bounds are their declared lengths. The corpus covers all six relations, prefixes, first and last differences, unterminated full arrays, and bytes beyond a shared terminator. Folded string comparisons and computed array comparisons agree.

CHAR case labels reuse the INTEGER range and overlap machinery after conversion to ordinals. Single labels, ranges, named CHAR constants, ordinal strings, reversed ranges, overlaps, wrong label types, and the no-match trap all pass their positive or negative gates.

The cross-module program imports a named character-array type, CHAR and string constants, and CHAR and BYTE variables. Its generated IL emits the client's own literal objects for the imported string bytes and refers directly to the declaring module's variable symbols. Imported character storage remains read-only in the client.

All seven positive programs were run directly. Each exited successfully, wrote an empty standard error stream, and produced byte-for-byte checked-in standard output.

## Declined changes

The review does not request string variables, `Out.String`, character-array parameters, or open arrays. Those remain assigned to later slices.

The review does not request a text encoding layer or character-set table. Treating valid UTF-8 source text as its original bytes is the deliberate representation choice in the plan.

The review does not request literal interning or scalarized string copies. One data object per runtime use and the existing byte-copy instruction are sufficient for this compiler.

The review does not cover the untracked Slice 12 plan.

## Verification

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, and `git diff --check` pass. The test run contains 32 unit tests and two integration tests.

`Chars`, `Bytes`, `Strings`, `StringCompare`, `CharCase`, `CharTable`, and `CharApi` were also run directly. Their standard output matched the checked-in expectations, and every standard error stream was empty.

The generated IL for `Chars`, `Bytes`, `Strings`, `StringCompare`, and `CharApi` was inspected for byte-sized globals and arrays, `loadub`, `storeb`, word-class calls and results, BYTE and CHR range checks, literal alignment and terminators, exact copy sizes, comparison bounds, and client-owned data objects for imported string constants.
