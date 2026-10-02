# Review: Records and structured parameters, as landed

A second review of Slice 12, read against commit `ddbf864` (`add records and structured parameters`) with a clean working tree. The first review, [2026-08-02-011](2026-08-02-011-records-and-structured-parameters.md), was written against the working tree before that commit and is not repeated here. This one reads the landed code independently, re-checks the evidence the first review recorded, and probes the paths that evidence did not reach.

## Verdict

The slice is correct as landed. Every item the [Slice 12 plan](../plans/2026-08-02-012-records-and-structured-parameters.md) promised is implemented and covered by a test, and nothing in the record, field, layout, copy, parameter, visibility, or constant-folding paths behaved wrongly under probing. Two documentation findings were confirmed and fixed. No behavioural finding was found, and no source or test file changed.

## Findings

### Low: A parser comment described a rule this slice removed

`Parser::formal_parameters` carried the comment "a fixed array formal is written as a qualident and sema rejects it". Semantic analysis accepted fixed array formals as of this slice, so the comment described the compiler as it was one slice earlier, immediately above the code that still rejects the open-array prefix. A reader checking whether array parameters work would have been told they do not.

Fixed: the comment now says that a structured formal is written as a named type and needs no syntax of its own, and that the open-array prefix is the one form still unsupported.

### Low: The architecture document stated the parameter convention without its reason or its visible consequence

The plan asked the architecture document to note that a value parameter of structured type is an address with a read-only rule, and why. The committed section states the convention and states that no callee-side copy is made. It does not say what licenses that reading of the Report, and it does not say that aliasing is therefore observable. Observable aliasing is the part a programmer notices, and `ParamAlias.Mod` exists to pin it, so the document should not be the one place that omits it.

Fixed: [agents/architecture.md](../architecture.md) now records why a structured value parameter is an address, namely that Report 10.1 confines the local-variable rule to basic types while Report 9.1 forbids assignment to a structured value parameter or its elements. It records the aliasing consequence in the same terms `ParamAlias.Mod` demonstrates. It also records that a string cannot be the actual for a fixed character-array formal, which is a permanent language rule the document did not mention at all.

## Verified implementation

Field addressing, layout, and copying were checked by compiling programs and reading both their output and their emitted IL.

A program with a record whose first field is a two-character inline record and whose second is an `INTEGER` laid the second field at offset four and gave the record size eight. A record holding an array of two records and a two-by-three array of `INTEGER` laid the second field at offset sixteen and gave the record size forty, and its whole-record copy passed forty to `oberon_copy`. Selecting through a record, an array of records, and a two-dimensional array field chained one `add` per field and one checked index per subscript, in source order. The program printed `xy   5`, then `q   8   9  10   7`, then `99`, then `3`, which is the expected value at every field after a whole copy followed by mutation of the source.

A record ending in a `BYTE` field after an `INTEGER` field was rounded from five bytes to eight, and an array of two of them occupied sixteen, which the copy sizes in the IL confirm. A variable declared through a type alias of that record assigned to and from the original type, confirming that an alias shares one identity.

Zero-size storage works in every position a record can occupy. A record whose only field is `ARRAY 0 OF INTEGER` has alignment four and size zero; its global emitted as `data $P3.ga = align 4 { z 0 }`, its locals as `alloc4 0`, and its assignments as `oberon_copy` with a count of zero. An empty record emitted `align 1 { z 0 }`. Both were passed as value and as `VAR` parameters and assigned in every direction without a QBE, assembler, or linker complaint.

Predefined operations reach record fields through the ordinary designator path. `INCL`, `EXCL`, `PACK`, and `UNPK` applied to `SET`, `REAL`, and `INTEGER` fields of a record reached through a `VAR` parameter, and printed `34  15   3`, the expected result of building the set `{1, 5}`, multiplying three by four, and splitting the product into a fraction and an exponent. `INC` of a `BYTE` field holding 255 exited unsuccessfully with `BYTE value out of range` on standard error, so the existing dynamic check runs unchanged from a field address.

Character-array fields keep the Slice 11 rules. A string assigned to a six-character field terminated inside the field, the terminator scan found length three, comparison of the field against a string and against another record's field gave the right answers in both directions, and a `CHAR` field drove a `CASE` statement with a range label and a single label. A `BYTE` field drove a `CASE` statement over integer labels.

Constant folding through fields works in every context that requires a constant. `LEN` of an array field, of a nested record's array field, and of one row of a two-dimensional array field all folded, as a procedure-local constant declaration, as the length of a procedure-local array type, and in an executable expression. The three forms printed `3   4   2` in both a procedure and a module body.

Type identity and visibility hold across module boundaries. A client that declares its own record type spelled exactly like an imported one is told `cannot assign Public to Public: these are different record types, and each RECORD in the source declares its own`, so the hint fires on the case it exists for. A record type declared in a procedure, used for a local, and passed by value and by `VAR` to a nested procedure works, and the nested procedure's constant declaration folds `LEN` of the value parameter's array field.

Diagnostics are the intended ones on the paths a program is most likely to take by mistake. An inline record in a formal parameter list is a parse error, `expected parameter type name, found Record`, because `FormalType` admits only a named type. A named record result type is reported as `a procedure cannot have the record result type R`. A record in `ASSERT`, in a `CASE` selector, and in arithmetic is reported as having a type that cannot be used as a value. `LEN` of a record is reported as an argument of the wrong kind. A duplicate field, an absent field, and a field selector applied to an `INTEGER` each get their own message.

The first review's own evidence re-checks correctly. `build/Params.ssa` still hashes to `bc6627e4199e252930401829769ea56477ae76987738f442f8111d8121233291`, which is the pre-slice value that review recorded, so code generation for programs written before this slice is unchanged. `ParamAlias` still prints `4  41` and `5  42`. `RecordLayout` reserves eight bytes for its padded record and twelve for its mixed one and copies exactly those counts. The zero-length-array record in `Records` emitted `align 4 { z 4 }`.

## Declined changes

The review does not ask for a copy of a structured value actual into the callee's frame. The read-only reference convention is a decision the plan reached against the Report and all three reference compilers, and the corpus pins its observable behaviour.

The review does not ask for record equality, extension, pointers, open arrays, or procedure types. The first three of those are the next slices' work, and equality is excluded by the Report rather than deferred.

The review does not ask for the plan's single `RecordBad.Mod` to absorb the extension diagnostic. The extension form is a parser error, which ends the parse, so the plan's own list of diagnostics in one file could not all have been reported. Splitting it into `RecordExtensionBad.Mod` is the only way to test both, and it is what the implementation did.

## Verification

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and `cargo test` pass, with 35 unit tests and two integration tests. The integration test compiles and runs every module in `tests/corpus`, `tests/errors`, and `tests/failures`, so the whole gate is inside that run.

The probe programs written for this review were deleted after it. Each one compiled, ran, and produced the output quoted above.
