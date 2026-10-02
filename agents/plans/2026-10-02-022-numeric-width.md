# Widen INTEGER, SET, and REAL to 64 bits

## Goal

`INTEGER` is a signed 32-bit integer, `SET` holds elements 0 through 31, and `REAL` is IEEE binary32. Array lengths, indices, and file positions share the 32-bit limit, and no Oberon type can hold a host address, a `long`, a `size_t`, or a `time_t`. The planned host-access slice needs an `INTEGER` that holds a C pointer and a `REAL` that is a C `double`, so the widths come first.

When this is done, `INTEGER` is a signed 64-bit integer, `SET` is a 64-bit vector holding elements 0 through 63, and `REAL` is IEEE binary64. `BOOLEAN`, `CHAR`, and `BYTE` keep their current storage. Every stage agrees on the new widths: lexer, constant folding, semantic checks, IR, QBE lowering, both runtime C files, the bundled modules, the tests, and the documentation.

## Related code

- `src/lexer.rs` — `Tok::Int(i64)` carries an integer literal so an oversized decimal still lexes; a hex literal is parsed with `u32::from_str_radix` and reinterpreted as a 32-bit pattern; `real` parses the literal text straight to `f32` and diagnoses an infinite result.
- `src/ast.rs`, `src/parser.rs` — `Expr::Int { value: i64 }`; the parser folds a leading minus into a bare literal so `MIN(INTEGER)` can be written, which is why the literal type must be wider than `INTEGER`.
- `src/sema/symbols.rs` — `ConstValue::Int(i32)`, `Real(f32)`, `Set(u32)`; `runtime_interface` names only source types and needs no change.
- `src/sema/constant.rs` — folds integers with checked `i32` arithmetic, reals at `f32`, and shifts and rotations modulo 32; `floor_const` uses the binary32 endpoints; `check_const_expr` range-checks literals with `i32::try_from`.
- `src/sema/mod.rs` — `SET_MAX`, `SET_FULL`, `FLOOR_MIN`, `FLOOR_LIMIT`; `lower_expr` range-checks literals; `lower_abs_int` compares against `i32::MIN`; `lower_shift` bounds counts at 31 and uses 32 as the rotation complement; `for_step`, `case_label`, `array_length`, `new_array`, and the range-error formatters use `i32`.
- `src/sema/types.rs` — `Type::scalar` maps source types to `ir::Ty`; `ArrayType.len: i32`.
- `src/ir.rs` — `Ty` and its width comment, `Value::Int(i32)`, `Set(u32)`, `Real(f32)`, `Storage::Array { len: i32 }`, `scalar_size` giving four bytes to `Int`, `Bool`, `Set`, and `Real`, `position` narrowing a source position to `i32`, `Site::args` passing positions as `Ty::Int`.
- `src/qbe.rs` — `class` puts `Int`, `Bool`, `Set`, and `Byte` in `w` and `Real` in `s`; `load_op` and `store_op` use `loadw`, `storew`, `loads`, `stores`; `bin_op` picks the `w` and `s` comparison forms; `value` spells set and real immediates as signed 32-bit patterns; `IntToReal` emits `swtof`; `Index` and `CopyElements` sign-extend word operands to `l` with `extsw`; `alloc` maps alignment to `alloc4`, `alloc8`, `alloc16`.
- `runtime/oberon.c` — every check, trap, and REAL helper takes `int32_t` and `float`; `oberon_floor` and `oberon_unpk` use the binary32 forms; `oberon_fail` prints positions with `%d`.
- `runtime/standard.c` — every `oberon_lib_` entry uses `int32_t` and `float`; `Out.Int`, `Out.Hex`, and `Out.Real` formats; `In.Int` accepts up to eight hex digits and the 32-bit decimal range; `In.Real` uses `strtof`; Math calls the `f` libm forms; Files transfers four bytes for `INTEGER`, `REAL`, and `SET`, clamps `Length` at `INT32_MAX`, and bounds the compact encoding at 32 bits.
- `lib/Out.Mod`, `lib/In.Mod`, `lib/Math.Mod`, `lib/Files.Mod` — Oberon wrappers; their logic is width-independent.
- `vendor/qbe/doc/il.txt` — subtyping: an `l` may be used in a `w` context, never the reverse; comparisons may define `w` or `l`; `loadub`, `loaduw`, `loadsw` define `l`; a `d` immediate may be written as a 64-bit integer bit pattern.
- `tests/corpus.rs` and the fixtures under `tests/corpus`, `tests/errors`, `tests/failures` listed in the test plan.

## Decisions

**Register classes.** `Int`, `Set`, `Byte`, and `Bool` travel in `l`; `Real` travels in `d`. Putting `Bool` in `l` keeps `ORD(BOOLEAN)` a pass-through and keeps every integer-like value in one class, so no extension instruction is added. Comparisons define `l` and `jnz` uses the value through subtyping. `Byte` loads with `loadub` into `l` and stores with `storeb`; `Bool` keeps four bytes of storage and uses `loaduw` and `storew`. Every hidden argument and every runtime C parameter of an integer-like type is therefore `int64_t`, and every `REAL` is `double`. The hard-coded `extsw` in `Index` and `CopyElements` lowering goes away because the operands are already `l`.

**Storage.** `scalar_size` gives eight bytes to `Int`, `Set`, and `Real`; `Bool` stays at four and `Byte` at one. Record layout and alignment follow from `size` and `align` with no other change. `MAX_OBJECT_SIZE` stays `1 << 30`: it bounds one object's size on the target, not the integer type, and `Storage::Array.len` becomes `i64` only because the length is an `INTEGER`.

**Literals.** `Tok::Int` and `Expr::Int` become `i128`, so the parser's sign folding still produces `-9223372036854775808`, and sema range-checks to `i64` with the existing `integer literal out of range` diagnostic. A hex literal is a 64-bit pattern: up to sixteen digits, parsed with `u64::from_str_radix` and reinterpreted as `i64`, so `0FFFFFFFFFFFFFFFFH` is `-1` and `0FFFFFFFFH` is `4294967295`. This follows the existing rule that a hex literal is a pattern of the `INTEGER` width, not a magnitude, and changes the meaning of eight-digit literals with the top bit set. A REAL literal parses once to `f64` and is diagnosed when the binary64 result is infinite.

**Constants.** `ConstValue::Int(i64)`, `Set(u64)`, `Real(f64)`; `ir::Value` the same. Folding keeps checked `i64` arithmetic and the `constant expression overflows` diagnostic. `SET_MAX` is 63 and `SET_FULL` is `u64::MAX`. Shift and rotation counts are 0 through 63 and the rotation complement is 64 with mask 63. `FLOOR_MIN` is `-9223372036854775808.0` and `FLOOR_LIMIT` is `9223372036854775808.0`, both exact in binary64, and `oberon_floor` uses the same pair. `FLT` emits `sltof`; above `2^53` the conversion rounds, which the existing comment on `Flt` already describes.

**Immediates.** A set immediate is its `u64` bits spelled as a signed `i64`. A real immediate is its binary64 bit pattern spelled as a signed `i64` in a `d` context, which `il.txt` shows as a legal spelling. This keeps negative zero, infinities, and NaNs exact as today.

**Library formats.** `Out.Int` prints the full 64-bit range. `Out.Hex` prints one space and sixteen uppercase hex digits. `Out.Real` prints `%.15E`: sixteen significant digits, the most that print without binary64 noise in the last place. `In.Int` accepts the 64-bit decimal range and one through sixteen hex digits. `In.Real` converts with `strtod`. Math calls the `double` libm forms, and `Math.pi` and `Math.e` keep their fifteen-digit spellings, now rounded to binary64. `Files` transfers eight bytes for `INTEGER`, `REAL`, and `SET`, and `Length` and positions are `int64_t` using `long`, which is 64 bits on the supported LP64 targets. The compact encoding keeps its byte format with continuation bytes carrying seven bits and the final byte six value bits and a sign; it accepts any encoding whose value fits `int64_t` and reports `invalid compact INTEGER encoding` for longer ones.

**Positions.** `ir::position` returns `i64`, `Site::args` is unchanged in shape, and `oberon_fail` casts to `long long` for `%lld`.

**C formatting.** `runtime/standard.c` casts to `long long` and `unsigned long long` for `%lld` and `%llX` rather than depending on `PRId64` spellings.

## Test plan

Fixtures whose expected output or diagnostics change with the widths:

- `tests/corpus/HexLit`: `0FFFFFFFFH` now prints `4294967295`; add `0FFFFFFFFFFFFFFFFH`, `08000000000000000H`, `07FFFFFFFFFFFFFFFH`, and the literal `-9223372036854775808`.
- `tests/corpus/LibraryOut`: `Out.Hex` shows sixteen digits, `Out.Real` shows fifteen fractional digits, and the `Out.Int` endpoints become the 64-bit ones.
- `tests/corpus/LibraryIn` and its `.stdin`: 64-bit decimal endpoints, a sixteen-digit hex token, a seventeen-digit hex token that fails, an out-of-range decimal that fails, and a REAL token that needs binary64 precision.
- `tests/corpus/LibraryFileEncoding`, `LibraryFiles`, `LibraryFileFailures`: eight-byte scalar encodings, `res` after a short read reporting the missing count out of eight, and compact numbers at both 64-bit endpoints.
- `tests/corpus/LibraryMath`: tolerances and identities at binary64.
- `tests/corpus/Reals`, `RealIeee`, `RealBuiltins`, `RealPacking`, `RealProcedures`: outputs re-pinned at binary64; `UNPK` exponents and `PACK` round trips at the new precision.
- `tests/corpus/Sets`, `Builtins`, `Bytes`: outputs involving `Out.Hex` or shift widths re-pinned.
- `tests/errors/BuiltinBad`, `SetInConstBad`, `SetElementConst`: diagnostics read `between 0 and 63`, and the offending constants move to 64 and above.
- `tests/errors/RealLiteralRange`: `1.8E308` and `1.0E309` are too large; `3.5E38` is now valid and moves to a corpus program.
- `tests/errors/FloorConstRange`: magnitudes around `1.0E19`; `FLOOR(FLT(MAX(INTEGER)))` still fails because `FLT` rounds up to `2^63`.
- `tests/failures/FloorRangeHigh`, `FloorRangeLow`, `ShiftRangeHigh`, `ShiftRangeLow`, `AbsOverflow`: inputs at the 64-bit boundaries, same messages.

New fixtures:

- `tests/corpus/WideIntegers.Mod`: products and sums past 32 bits, `DIV` and `MOD` on large negatives, `LSL(1, 63)`, `ASR` of `MIN(INTEGER)`, `ROR` by 40, `ABS` of a value past 32 bits, `FLOOR(1.0E15)`, and `FLT(9007199254740992)` printed exactly.
- `tests/corpus/WideSets.Mod`: elements 32 through 63, ranges crossing 32, complement, `ORD` of a set with element 63 printing a negative `INTEGER`, `INCL` and `EXCL` at 63, and `IN` at both ends.
- `tests/corpus/RecordLayout.Mod` gains a record mixing `BOOLEAN`, `CHAR`, `INTEGER`, and `REAL` fields so the eight-byte alignment is observable through array copies and `Files.WriteBytes`.
- `tests/errors/IntegerLiteralRange.Mod`: `9223372036854775808` and a seventeen-digit hex literal.
- `tests/failures/SetElementRange.Mod` if none exists: a dynamic element 64.

Every corpus program compiles with no standard error, exits zero, and matches its expected output byte for byte. Programs with no `REAL`, `SET`, hex output, or shift keep their existing expected files.

## Implementation plan

1. Lexer and AST: `Tok::Int(i128)`, `Tok::Real(f64)`, `Expr::Int { value: i128 }`, `Expr::Real { value: f64 }`. Parse hex literals with `u64::from_str_radix` into an `i64` pattern. Parse REAL literals to `f64`. Update the lexer unit tests for the largest accepted and smallest rejected REAL literal and add a sixteen-digit hex case.
2. Semantic types and constants: widen `ConstValue`, `ArrayType.len`, `SET_MAX`, `SET_FULL`, `FLOOR_MIN`, `FLOOR_LIMIT`, `floor_const`, shift and rotation folding, every `i32::try_from` on literals, `for_step`, `case_label`, `array_length`, `new_array`, `lower_abs_int`, `lower_shift`, and the range-error formatters. Change the diagnostics to `between 0 and 63`.
3. IR: widen `Value`, `Storage::Array.len`, and `position`; set `scalar_size` to eight for `Int`, `Set`, and `Real`; rewrite the `Ty` comment to describe the new classes and widths.
4. QBE lowering: `class`, `load_op`, `store_op`, `bin_op`, `value`, and `IntToReal`. Remove the `extsw` lines in `Index` and `CopyElements`, and the `%.i`/`%.x` split they needed. Confirm `alloc8` is chosen for eight-byte scalars.
5. `runtime/oberon.c`: `int32_t` to `int64_t`, `float` to `double`, the `f` libm forms to their `double` forms, the `oberon_floor` endpoints, and `oberon_fail` formatting.
6. `runtime/standard.c`: the same type changes throughout; `Out.Int`, `Out.Hex`, `Out.Real`, `In.Int`, `In.Real`; Math; eight-byte Files transfers and `Length`; the compact encoding at 64 bits.
7. Run `make` so the archive and staged modules are rebuilt, then `cargo test`. Update the changed expected files by inspecting each new value against the decision it follows, not by copying output blindly.
8. Add the new fixtures from the test plan.
9. Documentation updates below.

## Documentation updates

- `AGENTS.md`: the Representation paragraph states the new widths and that `INTEGER` holds a host address.
- `docs/standard-library.md`: `Out.Int`, `Out.Hex`, `Out.Real`, `In.Int`, `In.Real`, Math's libm forms and constant rounding, the Files eight-byte encodings and compact-encoding bound, and the Target dependence section.
- `agents/roadmap.md`: reduce the Numeric width bullet to the remaining fact that `INTEGER` overflow wraps without a trap, and drop the Scope sentence saying `SYSTEM` needs a separate address representation.
