# Widen INTEGER, SET, and REAL to 64 bits

## Plan

`agents/plans/2026-10-02-022-numeric-width.md`

## Summary

`INTEGER` is signed 64-bit, `SET` holds elements 0 through 63, and `REAL` is binary64 in the lexer, constant folding, sema, IR, QBE lowering, both runtime C files, the bundled modules, the tests, and the documentation. The plan's goal is met.

## Departures from the plan

- `lib/Files.Mod` was not width-independent: `ReadInt`, `ReadReal`, `ReadSet`, `WriteInt`, `WriteReal`, and `WriteSet` computed `res` as `4 - moved`. They now use `8 - moved`.
- `oberon_lib_file_read_bool` keeps an `int32_t *` destination, because a `BOOLEAN` variable keeps four bytes of storage even though its value travels in `l`.
- `oberon_type_test_pointer` and `oberon_type_test_descriptor` return `int64_t` and are called as `=l`, since `BOOLEAN` now travels in `l`.
- Lexer diagnostics stop compilation before sema, so one module cannot show both literal errors. `tests/errors/IntegerLiteralRange.Mod` covers the decimal case and `tests/errors/HexLiteralRange.Mod` covers the seventeen-digit hex case.
- `tests/corpus/RecordLayout.Mod` shows the mixed `BOOLEAN`, `CHAR`, `INTEGER`, `REAL` record only through a whole-array copy. Without `SYSTEM`, a record cannot be passed to `Files.WriteBytes`, so byte offsets cannot be observed from Oberon.
- `tests/failures/SetElementRange.Mod` was not added: `SetElementHigh` already exists and now uses element 64.
- The three object-size fixtures (`ArrayFrameTooLarge`, `ArrayModuleStorageTooLarge`, `RecordObjectTooLarge`) needed their array lengths halved. Their arrays of `INTEGER` doubled in size, so the inner type itself crossed the limit.
- `3.5E38` moved into `tests/corpus/Reals.Mod` as a new final row.

## Decisions

- The compact decoder accepts at most nine continuation bytes. After nine, the final byte stands for bit 63 alone, so only a sign of 0 or −1 is valid there. The value is formed in unsigned arithmetic. `LibraryFileFailures` covers both rejections.
- `In.Int` checks the decimal bound before each multiply-add, so the `uint64_t` accumulator never wraps on long tokens.
- The roadmap bullet is now titled **Integer overflow**.

## Checks run

- `make test` — 43 unit tests and 6 integration tests pass.
- `cargo fmt --check` — Clean.
- `cargo clippy --all-targets -- -D warnings` — Clean.

## Manual verification

1. Every regenerated expected file was checked by hand against the 64-bit rules before it was recorded. For example, `{0, 3 .. 4, 63}` is 25 − 2⁶³, `ROR(1, 40)` is 2²⁴, `UNPK` of 2⁻¹⁰⁵⁰ gives exponent −1050, and the folded and runtime columns agree in every two-column fixture.

   ```sh
   make && for f in HexLit Builtins Sets SetRanges WideIntegers WideSets RealBuiltins RealPacking RecordLayout LibraryOut; do target/debug/oberon tests/corpus/$f.Mod -o /tmp/$f && /tmp/$f; done
   ```

   The output matches each `.expected` file.

2. Every error fixture whose numbers changed reports the same line and column as before.

   ```sh
   for f in BuiltinBad SetElementConst SetInConstBad RealLiteralRange FloorConstRange; do target/debug/oberon tests/errors/$f.Mod -o /tmp/x; done
   ```

   Only the numbers and `between 0 and 63` changed.
