# Review: Scalar control and predefined operations

Reviewed against the uncommitted working tree on top of commit `1616ffd`
("docs"). The implementation follows
[the Slice 6 plan](../plans/2026-08-02-006-scalar-control-and-predefined-operations.md).

## Verdict

The compiler implementation is complete against the plan. The implemented
behavior agrees with Report sections 9.5, 9.8, and the applicable parts of
10.2. The generated control flow also follows the decisions recorded in the
plan.

One medium-priority test finding remains. The successful-program corpus does
not require the generated executable to exit successfully. This weakens the
new test for a successful `ASSERT` and can hide other traps after the expected
output has been written.

## Finding

### Successful corpus programs can exit with failure and still pass

The first loop in the `corpus` test runs each module under `tests/corpus/`.
It compares only standard output. It never checks `run.status`, and it never
checks standard error.

This matters to this slice because `Builtins` ends with `ASSERT(n = 7)`. A
reversed assertion branch would print every expected line and then fail.
The corpus test would still pass.

A temporary `ReviewExit` corpus module confirmed the gap. Its body was:

```oberon
Out.Int(7, 0); Out.Ln;
ASSERT(FALSE)
```

Its expected output was:

```text
7
```

`cargo test --test corpus corpus` reported success. Running the generated
program directly exited with status 1. It wrote `7` and a newline to standard
output. It wrote the following line to standard error:

```text
assertion failed
```

The probe was removed after the check. The corpus test should report a failure
when a successful-program executable has a nonzero status. Its failure message
should include both captured streams, because either stream may explain the
exit.

## Verified implementation

The parser now recognizes both remaining statement forms. A `FOR` statement
requires a bare control identifier. A numeric `CASE` statement accepts the
Report's integer and qualified-identifier label forms. Direct negative labels
and label expressions remain parse errors as planned.

`lower_for` stores the initial value before the first test. The limit expression
sits inside the loop and is evaluated before every test. `ForLimit` prints four
limit calls for three iterations and one call for an initially empty loop.
Assignments, variable arguments, and `INC` or `DEC` calls all reject the
read-only control variable inside the loop body.

`lower_case` evaluates the selector once. It compares that value against every
label range. The label table rejects reversed and overlapping ranges across the
whole statement. A labelled arm with an empty body exits normally. A selector
with no matching label calls `oberon_case_no_match` and halts.

The predefined identifiers now live below the module scope. Module declarations
can shadow them without colliding with them. The existing `Params` module still
has SHA-256
`bc6627e4199e252930401829769ea56477ae76987738f442f8111d8121233291`, which is
the hash recorded before the scope change.

Constant folding and runtime lowering agree for the new function procedures.
The checks covered negative `ODD`, signed right shifts, rotations of negative
values, rotation by zero, and counts at both valid boundaries. They also covered
logical left shift values that discard high bits. Runtime `ABS` rejects
`MIN(INTEGER)` before negation.

`INC` and `DEC` accept one or two arguments. They work on globals, locals, value
parameters, and variable parameters. An additional probe used side-effecting
step expressions. Each expression ran once, and the target was updated from the
value present after the expression returned.

`ASSERT` branches to the new runtime trap only when its Boolean argument is
false. Each of the six new failure programs exited with status 1 and wrote its
expected diagnostic. The two shift failures covered counts of -1 and 32.

The generated IL has the intended shape. `ForLimit.ssa` calls `Limit` between
the loop test label and the comparison. `Case.ssa` calls `Selector` once before
the comparison chain. Each case chain ends with `oberon_case_no_match` followed
by `hlt`. The dynamic shift guard and the absolute-value guard both precede the
operation they protect.

## Declined changes

The review did not request a jump table for numeric cases. The comparison chain
is simpler and matches the project's optimization goals.

The review did not request overflow checks for `FOR`, `INC`, or `DEC`. Ordinary
integer addition and subtraction do not yet have a runtime overflow policy.
Adding checks only to these operations would create a new inconsistency.

The review did not request source positions in runtime traps. The runtime still
has no emitted source-file string to include in those messages.

## Verification

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`,
and `git diff --check` pass. The five successful gate programs were also run
directly. Every program exited with status zero and wrote nothing to standard
error.

The six new failure programs were run directly. Every program exited with
status one and produced the exact expected standard error. All new negative
modules failed compilation with the diagnostic counts recorded in their
expected files.
