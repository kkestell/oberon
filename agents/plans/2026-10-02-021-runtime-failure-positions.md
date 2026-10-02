# Runtime failure positions and output ordering

## Goal

A runtime failure prints a fixed message such as `array index out of bounds`
with no source position, and it writes that message to stderr before the
buffered stdout is flushed by `exit`, so on a shared terminal the failure
appears ahead of output the program wrote earlier. After this change, every
language failure flushes stdout first and then prints
`Module:line:column: message` to stderr, where the position is the failing
check's position in the module's source, and still exits with status 1.

## Related code

- `runtime/oberon.c` — the check entry points (`oberon_check_nil`,
  `oberon_check_procedure`, `oberon_check_index`, `oberon_check_array_copy`,
  `oberon_floor`, `oberon_unpk`) and the traps (`oberon_div_by_zero` through
  `oberon_chr_range`) each `fputs` a fixed message to stderr and `exit(1)`.
  The comment above the traps already promises positions in a later slice.
- `src/ir.rs` — `Inst::CheckNil`, `CheckProcedure`, `Index`, and
  `CheckArrayCopy` carry no position. A trap is `Inst::Call` with no
  arguments followed by `Inst::Halt`, and `Halt` has no other use.
  `Literal` holds the string data objects of a module; `Value::Symbol`
  lowers to a data address.
- `src/sema/mod.rs` — `trap(symbol)` emits the call and halt pair.
  `literal(bytes)` creates one data object and returns its `Addr::Global`.
  Every site that emits a check has a `Pos` in scope or one parameter away:
  `dereference(base, pos)`, `index(base, expr, pos)`, `lower_call(.., pos)`,
  `lower_assign(lhs, rhs)` called from `Stmt::Assign { pos }` whose `pos` is
  marked `dead_code`, `copy_string(.., pos)`, `lower_case(expr, arms)`,
  `check_set_element(expr, value)`, `check_byte_domain(expr, value, domain)`
  with `expr` absent for `INC` and `DEC` in `lower_inc_dec(.., pos)`,
  `div_zero_check(divisor)` called from `lower_binary(.., pos)`,
  `lower_abs_int(arg)` and the `FLOOR` and `UNPK` lowering in
  `lower_builtin(.., pos)`, `lower_shift(.., count_expr)`,
  `lower_assert(.., pos)`, and `emit_guard_check(test)` called from
  `guard_place(.., pos)`.
- `src/qbe.rs` — `emit_inst` writes each check call by hand; the `Call` arm
  formats `ir::Arg` values inline. `Names::new` registers every literal
  symbol, so a literal's address is `$g<n>` in QBE. `emit_proc` treats
  `Halt` as a terminator.
- `src/driver.rs` — `fail` prints compile diagnostics as
  `path:line:col: message`; the runtime format follows it.
- `runtime/standard.c` — `Out` writes through `putchar` and `printf`, so
  stdout is the C stream and `fflush(stdout)` empties it.
- `tests/corpus.rs` — a `tests/failures` case compares the program's stderr
  exactly against its `.expected` file. `program_arguments_and_exit_status`
  is the model for a dedicated test that writes its own module.
- `tests/failures/*.expected` — sixty files holding the bare messages.

## Decisions

**Format.** A failure prints one line, `Module:line:column: message`, with
the module name rather than a file path: the executable does not know where
its source was, and the module name is what the compiler and the Report call
the unit. The line and column are 1-based, as in compile diagnostics. The
messages themselves do not change.

**Which position.** Each check reports the position its compile-time
diagnostic for the same construct would use, so a folded and a dynamic
failure point at the same place: the index expression for an index check
(so `a[i, j]` tells the two apart), the `^` or field selector for a nil
dereference, the call for a nil procedure call, `ASSERT`, `ABS`, `FLOOR`,
`UNPK`, `CHR`, `INC`, and `DEC`, the shift count expression for a shift,
the element expression for a `SET` element, the stored expression for a
`BYTE` store, the operator for `DIV` and `MOD`, the selector expression for
a `CASE` without a match, the guard selector for a type guard, and the
assignment for an array copy.

**Bundled modules.** A violated precondition in `Files`, `Strings`, or
`Program` is an `ASSERT` in that module, so it reports the position of that
`ASSERT` inside the bundled source, for example `Files:52:5: assertion
failed`. The program has no caller position to offer, and the `ASSERT` names
which precondition failed. The `tests/failures/Files*.expected` and
`Program*.expected` files therefore encode library line numbers and change
when the library does.

**Carrying the position.** `ir::Site` names the data a failing check
reports:

```rust
// Where a runtime check came from: the data object holding the module name
// and the 1-based position in that module's source. The runtime prints both
// when the check fails.
pub struct Site { pub module: String, pub pos: Pos }
```

`Site::args()` yields the three `ir::Arg`s the runtime receives, in order: the
module name as `Arg::Val(Ty::Pointer, Value::Symbol(module))`, then the line
and column as `Arg::Val(Ty::Int, Value::Int(..))`. `CheckNil`,
`CheckProcedure`, `Index`, and `CheckArrayCopy` gain a `site: Site` field. A
new `Inst::Trap { symbol: String, site: Site }` replaces the `Call` and `Halt`
pair, and `Halt` is removed. `FLOOR` and `UNPK` stay `Inst::Call`s and append
`site.args()` to their argument lists. Every QBE check call ends with the same
three operands, formatted by the one `arg` function the `Call` arm already
uses.

**The module name object.** Sema creates the name through `literal()` once
per module, when the module's analysis begins, and keeps its symbol in a
`site_module` field; `site(pos)` builds a `Site` from it. The literal is
emitted and renamed exactly like a string literal, so `Names` needs no new
rule. The `Literal` comment in `src/ir.rs` notes that the module name is one
of the literals.

**Runtime.** One static `oberon_fail(message, module, line, col)` does the
work: `fflush(stdout)`, one `fprintf` to stderr, `exit(1)`. Every check and
trap gains the trailing `const char *module, int32_t line, int32_t col`
parameters and calls it. Flushing stdout before writing stderr is what fixes
the ordering; `exit` already flushes the other streams, including `Files`
streams.

## Test plan

- Regenerate all sixty `tests/failures/*.expected` files in the new format and
  check each position by eye against its `.Mod` source: every line and column
  must point at the construct named under "Which position". The `Files*` and
  `Program*` cases point into `lib/Files.Mod` and `lib/Program.Mod`.
- New `tests/corpus.rs` test `failure_follows_earlier_output`: write a module
  `OutputOrder` to a fresh temporary directory that prints `before` and a line
  feed with `Out`, then indexes `ARRAY 2 OF INTEGER` with a variable holding
  2. Compile it with the shared `compile` helper and run it through
  `sh -c 'exec "$0" 2>&1' build/OutputOrder` so both streams share one pipe.
  The combined output is exactly `before\nOutputOrder:L:C: array index out of
  bounds\n` with the index expression's position, and the exit code is 1.
- The existing `tests/corpus` and `tests/errors` cases still pass, and
  `program_arguments_and_exit_status` still sees an empty stderr.

## Implementation plan

1. `src/ir.rs`: add `Site` and `Site::args()`, add `site` to the four check
   instructions, add `Inst::Trap`, remove `Inst::Halt`, and extend the
   `Literal` comment.
2. `src/sema/mod.rs`: add the `site_module` field, create the module name
   literal at the start of analysis, add `site(pos)`, change `trap(symbol)` to
   `trap(symbol, pos)` emitting `Inst::Trap`, and thread a `Pos` into
   `lower_assign` (dropping the `dead_code` allowance on `Stmt::Assign`),
   `check_byte_domain`, `div_zero_check`, `lower_abs_int`, and
   `emit_guard_check`. Pass the chosen position at every site listed under
   "Which position", including the `FLOOR` and `UNPK` call arguments.
3. `src/qbe.rs`: extract the `Call` arm's argument formatting into `arg`,
   append `site.args()` to the four check calls, lower `Trap` to the call
   followed by `hlt`, and treat `Trap` as a terminator in `emit_proc`.
4. `runtime/oberon.c`: add `oberon_fail`, give every check and trap the three
   position parameters, route each through `oberon_fail`, and replace the
   "later slice" comment with one explaining the flush and the format.
5. `tests/failures/*.expected`: regenerate and verify. `tests/corpus.rs`: add
   `failure_follows_earlier_output`.

## Documentation updates

- `docs/standard-library.md`: in the opening paragraph, state that a language
  runtime failure flushes standard output, prints
  `Module:line:column: message` to standard error, and ends the program with
  status 1, and that a violated precondition of a bundled module reports the
  position of the `ASSERT` inside that module.
- `AGENTS.md`: in the Runtime paragraph, "Language failures terminate the
  program" becomes "Language failures report their source position and
  terminate the program".
- `agents/roadmap.md`: remove the "Runtime failure positions" and "Output
  ordering" items.
