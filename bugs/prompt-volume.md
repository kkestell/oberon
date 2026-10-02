# Bug hunt at volume: the Report as the only oracle

You are hunting for bugs in an Oberon-07 compiler. You have the compiled compiler, the language
definition, and nothing else. Your job is to write and run as many small Oberon-07 programs as you
can, and to record every case where the compiler disagrees with the language. You are not fixing
anything, and you are not reading the compiler. You are building a triage queue.

Work from `/Users/kyle/src/oberon`. Every command below assumes it is your working directory.

Volume is the point of this run. A previous hunt wrote ten careful programs and filed two findings.
This one should write several hundred. Aim for at least 300 distinct programs, in batches, and keep
going while you are still learning something.

## What you may read

Exactly two directories:

- `references/` — the language definition and the grammar.
- `bugs/` — this file, and any findings a previous run left behind.

Everything else in the repository is off limits, including `src/`, `docs/`, `tests/`, `lib/`, and
`runtime/`. Do not open them, do not grep them, do not list them looking for hints. Those
directories still have to exist for the compiler to work, and you will invoke code that lives in
them; you just may not read them.

`bugs/prompt.md` is a different run's instructions. Ignore it. It points at documentation you are
not using, and following those pointers defeats the purpose of this run.

The point of the restriction is to find out what the Report alone catches. If you cannot justify a
finding from the Report, the grammar, or the compiler contradicting itself, it is not a finding
here, however strong your instinct.

## Ground truth

The normative definition is Wirth's *The Programming Language Oberon (Revised Oberon-07)*, May 2016,
at `references/oberon07-report.pdf`, with an extracted grammar beside it at
`references/oberon07-grammar.ebnf`. Read the Report before you start. Extract it to text so you can
search it — `pdftotext` is available — and cite section numbers in your findings.

Read `bugs/INDEX.md` and the existing finding directories before you write anything. They are your
only prior art. Do not re-file what is already there.

## Setup

The compiler is already built at `./target/debug/oberon`, with its QBE, runtime, and bundled
modules staged in `target/lib/oberon/`. Do not build it. Do not run its test suite. If either is
missing, stop and report that.

Without `-o`, the compiler writes the executable to the working directory, named after the module.
Every command below passes `-o build/<Module>` so executables stay out of the repository root.
Create `build/` first with `mkdir -p build`; the compiler does not create it.

Leave `RUST_BACKTRACE` unset. The compiler reports ordinary source diagnostics through the same
error path as internal failures, so backtraces bury the one useful line under twenty-five lines of
stack, and you cannot read the source they refer to anyway. If a compile produces a Rust panic
message, copy it verbatim into the finding and move on.

## Calibrate the harness first

Before you trust any result, run three programs by hand and confirm the harness does what you think.

1. A program that prints a known value. Confirm it compiles, runs, exits zero, and prints it.
2. A program ending in `ASSERT(TRUE)`. Confirm it exits zero.
3. The same program with `ASSERT(FALSE)`. Confirm it exits nonzero, and note the exact message and
   which stream it goes to. Every later program depends on a failed assertion being detectable.

## Printing

`Out` is the only library module you may use. The Report defines no standard library, so you have no
specification for the others and cannot tell a bug from a design choice in them. Leave `In`, `Math`,
`Strings`, and `Files` alone entirely — a finding against an unspecified module is unfileable.

`Out` has this interface:

```oberon
PROCEDURE Open;
PROCEDURE Char(ch: CHAR);
PROCEDURE String(s: ARRAY OF CHAR);
PROCEDURE Int(i, n: INTEGER);
PROCEDURE Hex(i: INTEGER);
PROCEDURE Real(x: REAL; n: INTEGER);
PROCEDURE Ln;
```

`Int` writes the number in decimal, padded on the left with spaces to a minimum width of `n`; a zero
or negative `n` adds no padding. `Ln` writes one line feed. `Hex` writes a space and then eight
uppercase hexadecimal digits holding the argument's bit pattern, which is how you inspect a value
without trusting decimal formatting. `Real` writes exponential notation with six fractional digits.
`String` stops at the first null character.

Treat `Out` as correct. It is not the target of this run, and you have no specification to hold it
to beyond the paragraph above.

## The oracle problem

You have no document telling you what this implementation chose where the Report leaves a choice.
The Report leaves the range of `INTEGER`, the largest `SET` element, the precision of `REAL`, and the
character set of `CHAR` to the implementation. You can discover all four by experiment in the first
few minutes, and you should. But **a finding may never rest on the implementation having picked a
particular one of those.** That `SET` holds no element 32 is a choice, not a bug.

What you can hold the compiler to, with no specification at all, is agreement with itself. This is
where nearly all of your findings will come from:

- **Constant versus runtime.** Compute one expression twice: once folded into a `CONST` declaration
  where the compiler evaluates it, once through variables it cannot fold. The two must agree. If
  `CONST c = <expr>` and the same expression over variables disagree, one of them is wrong, and you
  do not need to know which to file it.
- **Round trips that hold by definition.** `CHR(ORD(c)) = c`. `FLOOR(FLT(i)) = i` wherever `FLT(i)`
  is exact. `x = (x DIV y) * y + (x MOD y)` for every `y` the Report defines. A value stored into a
  variable and read back. A record assigned to a base and its prefix compared field by field.
- **Two paths to one value.** `a[i][j]` against `a[i, j]`. `INCL(s, n)` against `s := s + {n}`. A
  procedure called directly against the same procedure called through a variable of procedure type.
  Nested `ARRAY N OF ARRAY M OF T` against `ARRAY N, M OF T`. The Report says these are the same
  thing, so the compiler saying otherwise is a finding whatever the underlying representation.
- **Boundaries either side.** Whatever limit you discover, test the last value inside it and the
  first value outside it. Inside must work. Outside must fail cleanly — a diagnostic if the compiler
  can see it, a runtime trap with a nonzero exit if it cannot. Outside quietly producing a number is
  the most valuable finding shape there is.

## Writing programs at volume

The throughput unlock is that a program checks itself. `ASSERT` turns a program into its own oracle:
no expected-output file to author, no comparison to make, no judgement at run time. The exit status
is the whole signal. Reserve printing for minimizing a failure you have already found.

Work in batches of twenty to forty programs:

1. Pick a theme. Write the batch into `bugs/work/`, one module per file.
2. Give every module a distinct name across the entire run — `T0001` through `T0999` works, and a
   short note of what each one covers goes in your tally. Executables land in a single flat `build/`
   directory keyed by module name, so two modules with one name silently overwrite each other and
   you will run yesterday's binary thinking it is today's.
3. Each program should hold many assertions on **one** theme, so that a failure already tells you
   where to look. Print a small integer before each group of assertions, so a trap tells you how far
   it got and you do not have to bisect from scratch.
4. Run the batch. Anything that fails to compile, or exits nonzero, is a candidate.
5. Triage the candidates. Most will be bugs in your own program.
6. Clear `bugs/work/` and write the next batch. Programs that pass are not kept.

A batch runner along these lines:

```
for f in bugs/work/*.Mod; do
  m=$(basename "$f" .Mod)
  if ! timeout 60 ./target/debug/oberon -o "build/$m" "$f" >"bugs/work/$m.log" 2>&1; then
    echo "COMPILE $m"; continue
  fi
  timeout 60 "./build/$m" >"bugs/work/$m.out" 2>&1 || echo "RUN $m status=$?"
done
```

Wrap both the compile and the run in `timeout`, so a hang shows up as a finding instead of a stalled
session. Generating families of programs with a script is encouraged — sweeping an operator against
a matrix of operand values is exactly the kind of thing that finds constant-folding bugs, and no
human needs to write those out by hand.

Keep a running tally in `bugs/RUNLOG.md`: one line per batch, giving the theme, how many programs it
held, and how many candidates it produced. That is the only record the passing programs leave, and
your final report depends on it.

## What counts as a finding

1. **Wrong answer.** A valid program compiles and runs but computes the wrong value, takes the wrong
   branch, or prints the wrong thing.
2. **Disagreement with itself.** Two routes to one value, as above, that produce different results.
   File these even when you cannot say which route is wrong.
3. **Toolchain failure.** QBE, the assembler, or the linker rejects what the compiler emitted, on
   input that is valid Oberon-07. You will see this as a compile that fails with a message that is
   plainly not a source diagnostic.
4. **Compiler crash.** A Rust panic, an internal error, or a hang, on input that is valid Oberon-07.
5. **Wrong rejection.** A valid Oberon-07 program refused with a source diagnostic. Cite the grammar
   production or the Report rule that makes it legal.
6. **Missing check.** Something the Report makes illegal that the compiler accepts and runs. An index
   outside its array that quietly reads memory. A value outside a type's range that is stored anyway.
7. **Generated program crashes.** Segfault, bus error, memory corruption, or a collector abort, from
   a program that should have run cleanly. A real failure exiting with status zero counts too.

## What does not count

- Any of the four implementation-defined parameters having a particular value. See above.
- The `SYSTEM` module. It is deliberately absent.
- One deliberate deviation from the Report, which you would otherwise file and which is intended: a
  nested procedure may call an enclosing procedure and use its constants and types, but may not read
  or write the enclosing procedure's variables or parameters. The Report's scope rule in §4 arguably
  permits it; this compiler rejects it on purpose to avoid needing static links.
- Readings of a genuinely ambiguous passage. Where the Report admits two readings and the compiler
  picks one consistently, that is a design decision you have no document to check. Whether two
  separately written `ARRAY 8 OF INTEGER` declarations are one type or two turns on reading §6.2 as
  identity or as shape, and real Oberon compilers split on it. Cases like that go in
  `bugs/questions/` with both readings stated, not in a numbered finding.
- Features from other Oberon dialects. Oberon-07 has no `WITH`, no `LOOP`/`EXIT`, no `LONGINT`, no
  `COPY`, no `MIN`/`MAX`, no `HALT`, and no variadic anything. Check the grammar before you assume a
  construct exists.
- Your own syntax slips. Three rules catch people out who are writing Oberon-07 from memory of other
  Pascal-family languages. `RETURN` is only the trailing clause of a procedure body, never a
  statement inside an `IF` — to return early, assign to a local and fall through to the one `RETURN`.
  A statement sequence uses `;` as a separator, so the statement before `RETURN` or `END` does not
  carry one. A `VAR` section separates names with commas, not semicolons.
- Wording, spacing, or column numbers in diagnostics.
- A resource limit hit by an absurd program. A frame that will not hold a million-element local array
  is not a language bug. A frame that will not hold a hundred integers is.

Cases you believe are wrong but cannot justify from the Report go in `bugs/questions/`, at lower
priority, in the same shape as a finding. So does valid-looking Oberon-07 that the compiler accepts
and you suspect it should not. Do not let an unresolvable case stall the run.

## Where findings go

Findings live in `bugs/`, one directory per finding, numbered in the order you find them. Continue
the numbering from the highest that already exists and never renumber.

```
bugs/
  003-<short-slug>/
    NOTES.md
    T0123.Mod             # the minimized reproducer, keeping its module name
    <extra .Mod files>    # only if the bug needs several modules
  questions/
    <same shape>
  work/                   # transient, cleared between batches
  INDEX.md
  RUNLOG.md
```

`INDEX.md` is one line per finding: number, one-sentence title, category, confidence. Keep it
current as you go, so an interrupted run still leaves something useful behind.

Before filing, minimize. Strip the program until removing anything else makes the symptom disappear.
Then run the minimized case twice — nondeterminism changes what the finding means. Then confirm the
program is legal Oberon-07 by checking the grammar, because most apparent bugs are invalid test
programs. When the symptom turns on a size, script the search for the exact threshold rather than
guessing, and then vary a second dimension to learn what the threshold is really counting.

`NOTES.md` for each finding:

```markdown
# <one-line title>

Category: wrong answer | disagreement with itself | toolchain failure | compiler crash |
          wrong rejection | missing check | generated program crash
Confidence: high | medium | low

## Reproduce

    ./target/debug/oberon -o build/T0123 bugs/003-<slug>/T0123.Mod
    ./build/T0123

## Expected

<what should happen, and the Report section or grammar production that says so. For a
self-disagreement, state the two routes and why the Report makes them equivalent.>

## Actual

<exact output, exit status, and stderr. Paste it; do not paraphrase it.>

## Notes

<optional: what the symptom is sensitive to, and the threshold if there is one.>
```

Write in the present tense, describing the bug as it is. Do not narrate your investigation.

## Rules

- Do not modify anything outside `bugs/` and `build/`.
- Do not read anything outside `references/` and `bugs/`.
- Do not build the compiler and do not run its test suite.
- Do not commit, stage, or push. `bugs/` stays untracked.
- Keep every scratch file inside `bugs/work/`. `build/` will fill with artifacts; that is fine.
- Do not speculate in a finding about which part of the compiler is at fault. You have not read it.

## Finishing

Stop after roughly 300 programs, or when a budget you were given runs out. Then report back with:

- How many programs you actually ran, and how many batches. Take the number from `RUNLOG.md` rather
  than estimating it.
- The findings, most severe first, one line each.
- The themes that turned up nothing, with the program count for each. At this volume that is the
  more valuable half of the report: it says where the compiler is solid, and it is only credible
  with the counts attached.
- What you could not test, and why. Be specific about anything the read restriction blocked, since
  that is what this run is measuring.
