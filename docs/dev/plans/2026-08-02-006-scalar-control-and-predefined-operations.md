# Slice: Remaining scalar control and predefined operations

## Context

After Slice 5 the compiler handles INTEGER and BOOLEAN programs with constants,
variables, procedures nested to any depth, `IF`, `WHILE`, and `REPEAT`. Two
statement forms are still rejected by the parser with "not yet supported: FOR
and CASE statements", and the language has no predefined procedures at all:
`ABS`, `INC`, and `ASSERT` are undeclared identifiers today.

This slice adds both statements and the predefined operations that apply to the
types that exist. After it, the single-module scalar language is complete except
for the basic types that later slices introduce.

Report sections implemented: §9.5 (case statements, in their INTEGER form),
§9.8 (for statements), and the parts of §10.2 that apply to INTEGER and
BOOLEAN operands, namely `ABS`, `ODD`, `LSL`, `ASR`, `ROR`, the BOOLEAN form of
`ORD`, `INC`, `DEC`, and `ASSERT`. The applicable part of §4's predefined list
gets a home in the symbol table. After this slice the parser recognizes every
statement form. `CASE` remains limited to INTEGER until later type slices add
its CHAR, record, and pointer forms.

## Decisions taken from the Report and the references

### The limit expression is evaluated for every loop test

Report §9.8 defines the for statement by rewriting it as a while loop:

> FOR v := beg TO end BY inc DO S END
> is, if inc > 0, equivalent to
> v := beg;
> WHILE v <= end DO S; v := v + inc END

In that rewriting `end` sits inside the while condition, so it is evaluated
for every test of the loop condition. All three vendored compilers do exactly
that.
Project Oberon's `ORP.StatSequence` records the loop top with `L0 := ORG.Here()`
*before* it parses the `TO` expression, so the code computing the limit is
emitted inside the loop and re-executed by the backward jump. oberonc's
`OJP.StatSequence` has the identical shape. OBNC writes a C `for` statement
whose condition contains the limit expression, which C re-evaluates each time
round.

So this compiler re-evaluates it too. The evaluation occurs before every loop
test, including the final test that fails. The difference is observable when
the limit is a function call. The gate therefore includes a limit function
that increments a counter and returns a separate, fixed limit. A three-iteration
loop calls that function four times. An initially empty loop still calls it
once.

Declined: hoisting the limit into a temporary before the loop. It is what most
readers expect from Pascal, it is what an optimizer would eventually do, and it
would make the count in that gate program read `1`. It also contradicts the
Report's own equivalence and all three references, and a program that depends on
the difference is relying on a side effect in a loop limit either way.

### The control variable is read-only inside the loop body

The Report says nothing about assigning to the control variable inside the body.
Project Oberon and oberonc both set `obj.rdo := TRUE` before parsing the body and
clear it afterwards, which makes any assignment to the control variable report
the ordinary read-only error. OBNC does not check.

Follow the two Wirth-derived compilers and reject the assignment. A loop whose
body moves its own control variable is either a bug or a while loop in disguise.
This also earns a piece of infrastructure that Slice 7 needs anyway: `Symbol::Var`
gains a `read_only` flag, which is what an imported variable will be marked with
when cross-module visibility arrives.

The flag is checked in the three places that write through a name: assignment,
a `VAR` actual argument, and the first argument of `INC` or `DEC`. A nested for
statement that reuses the same control variable is rejected as a consequence,
because the inner statement finds its control variable already read-only. That
is Wirth's behavior as well, and a negative test pins it.

### A step of zero is an error

`BY` takes a constant expression (§9.8, and the grammar spells it
`ConstExpression`). Project Oberon reports "zero increment" when the step folds
to zero; OBNC only warns and generates an infinite loop. Report the error. A
step of zero cannot terminate, and this project has no warning level to put it
in.

### A case with no matching label fails at runtime

Report §9.5 defines what happens when a label matches and is silent about what
happens when none does. Oberon-07 has no `ELSE` in a case statement, so the
question cannot be dodged. Project Oberon leaves the numeric case unimplemented
("numeric case not implemented") and gives no guidance. oberonc emits a trap
carrying "Invalid case in CASE statement". OBNC emits `OBNC_CT`, which raises its
`CASE_EXP_MATCH` exception and exits.

Both compilers that implement the statement treat a missed case as a runtime
failure, so this compiler does the same. Falling through silently is the other
plausible reading, and it turns a typo in a label into a statement that quietly
does nothing.

### Case labels follow the grammar and must not overlap

The grammar allows an integer literal, a string literal, or a qualified
identifier as a label. It does not allow an arbitrary constant expression.
oberonc accepts expressions such as `Max - 1`, but that is an extension to the
normative grammar. This compiler does not adopt it.

For the INTEGER form implemented here, the parser accepts an integer literal or
a qualified identifier in each label position. A qualified identifier must
resolve to an INTEGER constant. A negative label can be written through a named
constant. Direct syntax such as `-1` or `Max - 1` is rejected because neither is
a `label` production. String labels remain unsupported until CHAR arrives in
Slice 11.

A reversed range is an error, following oberonc's "illegal value of constant"
check. An overlap between any two ranges anywhere in the statement is an error,
following oberonc's ordered label table and its "case label defined more than
once". Both checks run over the whole statement, not just within one arm.

An alternative with no labels is legal in the grammar (`case = [CaseLabelList
":" StatementSequence]`). `CASE k OF | 1: x := 1 END` therefore parses and the
empty alternative contributes no arm. `CASE k OF END` also parses and always
fails at runtime. This is distinct from a labelled arm whose statement sequence
is empty. A matching labelled arm performs no action and then exits the case
statement normally.

### Shift counts must be between 0 and 31

`LSL`, `ASR`, and `ROR` take a count that the Report never constrains. OBNC warns
about an out-of-range count when it can evaluate the count while compiling. It
still passes a dynamic count straight to C, where a count of 32 or a negative
count is undefined behavior. oberonc inherits the JVM's masking to five bits.
Project Oberon inherits the RISC processor's behavior.

QBE also reduces a shift count modulo the result width. The explicit check is
therefore required even though the backend itself gives out-of-range counts a
defined meaning.

None of those is a language rule, and the roadmap already commits to the opposite
approach one slice later: Slice 8 adds "a runtime check for out-of-range dynamic
elements rather than allowing target shift behavior to define the language". Use
the same rule here, one slice earlier, on the operation that raises the question
first. A count outside 0 to 31 is a diagnostic when lowering already knows its
value. This includes integer literals and named constants, because both lower to
an immediate IR value. Other count expressions get a runtime check. Constant
folding uses the same bounds.

The runtime check is skipped for a known count in range. No extra constant
folding pass is added for compound runtime expressions. The gates cover both
ends of the rule with counts of -1 and 32. They also cover the valid boundary
counts 0 and 31.

Declined: masking the count to five bits, as QBE, the JVM, and the RISC
processor do. It is free, it is defined, and it silently turns `LSL(x, 32)`
into `x`, which no reading of "logical shift left by n bits" supports.

### `ABS` checks overflow and `LSL` discards high bits

`ABS(MIN(INTEGER))` has no representable result. Constant folding rejects it
with the existing "constant expression overflows" diagnostic. Runtime lowering
checks for `MIN(INTEGER)` and calls `oberon_abs_overflow` before it negates the
argument. This keeps the folded and runtime domains aligned, as the Slice 6
roadmap requires.

Declined: inheriting the wrapping behavior of runtime unary minus. Unary minus
has that known inconsistency today, but extending it to a new operation would
make the roadmap's explicit constant/runtime gate fail.

`LSL` is different, and deliberately so. The Report calls it a logical shift, so
bits shifted past the top are discarded rather than overflowing. `LSL(1, 31)`
folds to `MIN(INTEGER)` and is not an error, even though the Report's gloss for
it reads "x * 2n" and the multiplication would overflow. Folding must not use
checked multiplication here.

### Predefined identifiers live in a new universe scope

Nine new names arrive with this slice, and `INTEGER` and `BOOLEAN` are already
in the symbol table. Today all of them sit in the module scope, which means a
module that declares its own `ABS` would get "'ABS' is already declared" rather
than shadowing the predefined one.

Report §4 forbids two objects sharing an identifier "within a given scope", and
the predefined identifiers are not declared in any module's scope. Wirth's
`ORB.Init` enters them into a scope called `universe` and then opens the module
scope inside it, and `ORB.NewObj` only checks the innermost scope for duplicates,
so a module-level redeclaration shadows rather than collides. oberonc has the
same arrangement.

So the scope stack gains a bottom entry holding the predefined types and
procedures, and the module scope moves up one index. This is the one change in
the slice that touches something Slice 5 built, and it needs care for that
reason. `resolve` rejects a variable found in a scope that is "neither the
innermost nor the module scope", and it identifies the module scope by index
zero. That index becomes one. The universe scope holds no variables, so the rule
itself is unchanged in meaning. The comparison changes from zero to one, and the
comment recording the invariant changes with it. Neither new statement pushes a
scope, which is what that comment predicted.

## What remains unsupported after this slice

- Export marks and imports other than `Out`, which are Slice 7. `TYPE`
  declarations arrive with arrays in Slice 10.
- Every type beyond INTEGER and BOOLEAN. That keeps out the predefined
  operations whose arguments cannot exist yet: `LEN`, `FLOOR`, `FLT`, `CHR`,
  `INCL`, `EXCL`, `NEW`, `PACK`, and `UNPK`. Those names stay undeclared rather
  than getting a "not yet supported" diagnostic of their own. Some valid calls,
  such as `ORD(CHR(65))`, can already be written syntactically. They remain
  unsupported until the result types they need are implemented.
- The CHAR form of case labels (Slice 11) and the record and pointer form of the
  case statement (Slice 15).
- The CHAR and SET forms of `ORD`, which arrive with those types.

One known limitation is introduced deliberately. An ascending for loop can fail
to terminate when its increment wraps near `MAX(INTEGER)`. A descending loop has
the corresponding problem near `MIN(INTEGER)`. The C reference reaches undefined
behavior. The RISC and JVM references wrap. Fixing only for-loop addition would
introduce an overflow rule that ordinary INTEGER addition does not yet have, so
this slice records the boundary rather than defining a partial overflow policy.

## Lowering

### For statements

The statement lowers to the Report's own rewriting, with the limit inside the
loop:

```text
        store beg -> v
@for.test:
        <limit expression>
        cond = v <= limit        (v >= limit when the step is negative)
        br cond, @for.body, @for.end
@for.body:
        <body>
        v = v + inc
        jmp @for.test
@for.end:
```

The comparison direction is chosen at compile time from the sign of the folded
step, so nothing tests the step at runtime. In loops that do not overflow, the
control variable holds the first value that failed the test. A gate program
prints it.

### Case statements

The case expression is evaluated once into a temporary, as §9.5 requires ("First
the case expression is evaluated"). Each arm then gets the same shape
`lower_guarded` already produces for an `ELSIF` chain: test, branch to the arm
body, fall through to the next arm's test. A single label compares for equality;
a range computes both bounds comparisons and combines them with a bitwise `and`,
which is sound because comparisons produce zero or one. Multiple ranges in one
arm branch to the same body label.

The chain ends in the failure block rather than at the end label:

```text
        call $oberon_case_no_match
        hlt
@case.end:
```

A jump table would be the classical lowering. It is an optimization, the roadmap
lists advanced optimization as a non-goal, and a comparison chain is what the
existing `IF` lowering already produces.

### Predefined operations

A new `Symbol::Builtin` variant carries a small enum, and `lower_call` handles it
before the user-procedure path, because these operations are generic or variable
in arity and none of them is a call in the emitted code.

- `ABS(x)` first checks for `MIN(INTEGER)`. It then tests the sign and branches
  to a negation or a copy, joining at a shared temporary. That is the same shape
  the short-circuit operators already emit, so QBE is known to accept it.
- `ODD(x)` is the truncating remainder compared against zero. It must *not* be
  compared against one: `-3 rem 2` is `-1`, so an equality test against one gets
  odd negative numbers wrong. Comparing against zero agrees with the Report's
  "x MOD 2 = 1" under the floored `MOD` this compiler already implements.
- `LSL`, `ASR`, and `ROR` become the new shift instructions. A dynamic count is
  checked before the instruction. `ROR` is a logical right shift combined with
  a left shift by `(32 - n) and 31`. The mask makes a rotation by zero the
  identity rather than a shift by the word width.
- `ORD(b)` emits no instruction. BOOLEAN values are already zero or one in a
  word, so the lowering returns the same value with the type changed to INTEGER.
- `INC` and `DEC` take a writable INTEGER designator and an optional INTEGER
  expression that defaults to one. They load, add or subtract, and store.
- `ASSERT(b)` branches on the condition to a failure block that calls the
  runtime and halts. No constant special case: `ASSERT(FALSE)` emits the test
  like any other and QBE folds it.

Every diagnostic these produce reuses the existing wording where one fits:
"argument 1 has type BOOLEAN, expected INTEGER" for a type mismatch, "wrong
number of arguments: expected 2, found 1" for arity, "argument 1 must be a
variable" for `INC` on a non-designator. `INC` and `DEC` need one new arity
message, because they accept one argument or two.

Constant folding covers `ABS`, `ODD`, `LSL`, `ASR`, `ROR`, and `ORD`. `eval_const`
currently rejects every call with "constant expression contains a procedure
call"; that message stays for user procedures and the builtin cases fold instead.
`INC`, `DEC`, and `ASSERT` are proper procedures, so in an expression they get
the existing "cannot be used as a value" diagnostic from either path.

## Changes by file

### src/ast.rs

`Stmt` gains `For` and `Case`. The for variant holds the control variable as a
`Designator` with no selectors, the start and limit expressions, an optional
step expression, and the body. The case variant holds the selector expression
and a list of arms. Each arm has a list of label ranges and a statement
sequence. Label bounds reuse `Expr`, but the parser constructs only integer
literal and qualified-name expressions in this position.

### src/lexer.rs

No change. `BY`, `TO`, `OF`, `FOR`, `CASE`, and the `..` token all exist already,
including the lookahead that keeps `1..2` from scanning as a real literal.

### src/parser.rs

Replace the "FOR and CASE statements" unsupported diagnostic with the two
statement parsers. The for parser takes a bare identifier, per the grammar, not a
designator. The case parser has a dedicated label parser so it does not accept a
general expression. It skips an alternative with no labels rather than recording
an empty arm. It still records a labelled arm whose statement sequence is empty.

### src/ir.rs and src/qbe.rs

`BinOp` gains `Shl`, `Shr`, `Sar`, `BitAnd`, and `BitOr`, printing as QBE's
`shl`, `shr`, `sar`, `and`, and `or`. Nothing else changes: the traps reuse
`Call` and `Halt`, and both statements are built from branches and labels that
already exist.

### src/sema.rs

- A universe scope holding the predefined types and procedures, with the module
  scope opened inside it, and the module-scope index in `resolve` updated from
  zero to one.
- `Symbol::Builtin`, plus a `Builtin` enum with nine variants.
- `Symbol::Var` gains `read_only`, checked in `addr_of`, in `var_actual`, and in
  the `INC` and `DEC` lowering. A helper sets the flag on the innermost binding
  of a name and returns the previous value, so the for statement can restore it
  rather than clearing it unconditionally.
- `lower_for` and `lower_case`.
- Builtin lowering in `lower_call`, builtin type checking in `check_const_call`,
  and builtin folding in `eval_const`.

### runtime/oberon.c

Four trap functions in the shape of the existing `oberon_div_by_zero`, each
writing one line to stderr and exiting with status one:

- `oberon_assert_failed`, writing "assertion failed".
- `oberon_abs_overflow`, writing "ABS overflows INTEGER".
- `oberon_case_no_match`, writing "CASE without matching label".
- `oberon_shift_range`, writing "shift count out of range".

None of them takes a source position. The existing trap does not either, and a
position is only worth adding once there is a way to name the source file in the
message, which needs string data the compiler cannot emit yet. Recorded as a
follow-up rather than done half way.

## New corpus modules

`tests/corpus/` (compile, run, compare stdout):

- `For.Mod` — ascending and descending loops, a step of three that overshoots the
  limit, a loop whose body never runs, two nested loops, and the value of the
  control variable after each loop has finished.
- `ForLimit.Mod` — the load-bearing one for the decision above. The limit
  function increments a module counter and returns a separate fixed value. A
  three-iteration loop prints a count of four because the final failing test
  also evaluates the limit. An initially empty loop prints a count of one.
- `Case.Mod` — single labels, several labels on one arm, ranges, negative labels,
  a label that is a named constant, an empty alternative, and a labelled arm
  whose statement sequence is empty. It also covers disjoint adjacent ranges,
  arms written out of order, and inclusive range endpoints. A case statement
  inside a loop takes several arms in one run. A selector function with a
  counter proves that the selector is evaluated once per case statement.
- `Builtins.Mod` — every new operation. Each function result is computed once
  by a constant declaration and once at runtime from a variable holding the
  same value. The two values are printed side by side, as `DivMod.Mod` already
  does for floored division. The module covers `ABS` of negative and positive
  values. It covers `ODD` of a negative odd value. It covers `LSL` at counts 0
  and 31, `ASR` of a negative value, and `ROR` at counts 0 and 31. It covers
  `ORD(TRUE)` and `ORD(FALSE)`. It exercises `INC` and `DEC`, with and without a
  step, on a module variable, a local, a value parameter, and a `VAR` parameter.
  It also executes a successful `ASSERT`.
- `PredefinedShadow.Mod` — module-level declarations shadow predefined types
  and procedures. A local declaration would not test the new universe scope,
  because local declarations can already shadow names in the current module
  scope.

`tests/failures/` (compile, run, expect a nonzero status and exact stderr):

- `AssertFail.Mod` — a failing `ASSERT` on a condition computed at runtime.
- `AbsOverflow.Mod` — runtime `ABS` on a variable holding `MIN(INTEGER)`.
- `CaseNoMatch.Mod` — a nonempty case statement whose expression matches no
  label.
- `CaseEmpty.Mod` — a case statement with no labelled arms.
- `ShiftRangeLow.Mod` and `ShiftRangeHigh.Mod` — shifts whose counts come from
  variables holding -1 and 32. Separate modules are needed because the first
  trap terminates the process.

`tests/errors/` (must fail with exact stderr):

- `ForBad.Mod` — a control variable of type BOOLEAN, a control variable that is
  a constant, a non-INTEGER limit, a step that is not constant, a step of zero,
  an assignment to the control variable inside the body, an `INC` on it, and a
  use of it as a `VAR` actual.
- `ForNested.Mod` — an inner for statement reusing the outer control variable.
  An assignment after the rejected inner loop proves that error recovery did
  not accidentally make the outer control variable writable.
- `CaseBad.Mod` — a BOOLEAN case expression, a label that is not constant, a
  named BOOLEAN constant used as a label, a reversed range, two overlapping
  ranges, and a single label repeated in a later arm.
- `CaseLabelExpression.Mod` — `Max - 1` in a label position. This is a parse
  error under the Report grammar even though oberonc accepts it.
- `BuiltinBad.Mod` — `ABS(TRUE)`, `ODD` on a BOOLEAN, a shift with a literal
  count of -1, a shift with a literal count of 32, and a shift with an
  out-of-range named constant. It also covers `ASSERT` with an INTEGER argument,
  `INC` on a constant, `INC` with three arguments, `INC` used in an expression,
  `ABS` called as a statement, and folded `ABS(MIN(INTEGER))`.

## Verification

1. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, and
   `cargo test`.
2. Inspect the IL, since the slice changes control flow. In `build/ForLimit.ssa`
   the call computing the limit must sit between the loop-top label and the
   branch, which is the greppable form of the re-evaluation decision. In
   `build/Case.ssa` the call computing the selector must appear once, ahead of
   the comparison chain. The chain must end in a call to
   `$oberon_case_no_match` followed by `hlt`. Inspect one dynamic shift and
   `ABS` to confirm each trap guard precedes the operation it protects.
3. Confirm an unchanged module still produces byte-identical IL. The universe
   scope moves every module-level name one index deeper in the scope stack and
   must not change a single emitted instruction; `Params.ssa` is the module the
   Slice 5 review used for this and has a recorded hash.
4. Run every new gate binary by hand and read the output, rather than trusting
   the expected files that were written alongside them.
5. Check the folded and runtime columns in `Builtins.Mod` agree row by row.

## Order of work

1. ast.rs and parser.rs: the two statements, and the removal of the unsupported
   diagnostic.
2. sema.rs: the universe scope, moving the predefined types into it, and the
   module-scope index change in `resolve`. This lands alone, with the
   byte-identical IL check, because it is the one step that can quietly disturb
   existing behavior.
3. ir.rs and qbe.rs: the shift and bitwise instructions.
4. sema.rs: the read-only flag, then the for statement.
5. sema.rs: the case statement, with its label table.
6. sema.rs: the predefined operations, lowering and folding together so the two
   rule sets are written side by side.
7. runtime/oberon.c: the four traps.
8. The corpus modules, plus a regression module for anything found on the way.
9. The verification list.
