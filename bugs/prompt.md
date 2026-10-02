# Bug hunt: Oberon-07 compiler

You are hunting for bugs in the Oberon-07 compiler at `/Users/kyle/src/oberon`. Your job is to
write small, valid Oberon-07 programs, predict what each one should do, run it, and record every
case where the compiler disagrees with the language. You are not fixing anything. You are building
a triage queue.

Work from the repository root. That directory is `/Users/kyle/src/oberon` and every command below
assumes it is your working directory.

## What counts as a finding

1. **Wrong answer.** A valid program compiles and runs but prints the wrong thing, or takes the
   wrong branch, or computes the wrong value.
2. **Compiler crash.** A Rust panic, a `todo!()`, an `unwrap` failure, an internal error, or a
   hang — on input that is valid Oberon-07.
3. **Toolchain failure.** QBE, the assembler, or the linker rejects what the compiler emitted, on
   input that is valid Oberon-07. The compiler exits nonzero and writes no executable. This is the
   thinnest ice in the project: the semantic core is in better shape than the code it hands
   downstream, and both of the last run's findings landed here.
4. **Wrong rejection.** A valid Oberon-07 program is refused with a source diagnostic.
5. **Runtime check wrong.** A check that should fire does not (an out-of-range index that quietly
   reads memory), or a check fires on legal code, or a real failure exits with status zero, or the
   message goes to stdout instead of stderr.
6. **Generated program crashes.** Segfault, bus error, silent memory corruption, or a garbage
   collector abort, from a program that should have run cleanly.

Record these separately, at lower priority, in `bugs/questions/`:

- Invalid Oberon-07 that the compiler happily accepts.
- Behavior you believe is wrong but cannot justify from the language definition.

## What does not count

- Documented implementation choices. `INTEGER` is signed 32-bit, `SET` is a 32-bit bit vector,
  `REAL` is IEEE binary32, `CHAR` and `BYTE` are one byte.
- The `SYSTEM` module. It is deliberately not implemented.
- Features from other Oberon dialects. Oberon-07 has no `WITH`, no `LOOP`/`EXIT`, no `LONGINT`, no
  `COPY`, no `MIN`/`MAX`, no `HALT`, and no variadic anything. If your test program uses one of
  those, the rejection is correct and the bug is in your program.
- Your own syntax slips. Three rules catch people out who are writing Oberon-07 from memory of
  other Pascal-family languages. `RETURN` is only the trailing clause of a procedure body, never a
  statement inside an `IF` — to return early, assign to a local and fall through to the one
  `RETURN`. A statement sequence uses `;` as a separator, so the statement before `RETURN` or `END`
  does not carry one. A `VAR` section separates names with commas, not semicolons. Each of these
  produces a parse error that reads like a compiler bug and is not one.
- Wording, spacing, or column numbers in diagnostics.
- Anything already asserted by a test in `tests/errors/` or `tests/failures/`. Those are intended
  behavior, and there are about 130 of them. Skim the filenames before you start.

## Ground truth

The normative definition is Wirth's *The Programming Language Oberon (Revised Oberon-07)*, May
2016. The repository expects it at `references/oberon07-report.pdf` with a grammar beside it at
`references/oberon07-grammar.ebnf`, but `references/` is gitignored and **may be missing from this
checkout — check first**. If it is there, read it and cite it. If it is not, you are relying on
your own knowledge of the language, so hold yourself to a higher bar: only report a finding when
you can state the rule you believe is being violated and would defend it.

For the bundled library modules in `lib/` (`Out`, `In`, `Math`, `Strings`, `Files`), the reference
is [docs/standard-library.md](docs/standard-library.md). That document, not your expectations from
other Oberon systems, defines what those procedures do.

Every finding needs a stated expected result and a reason that expectation is right. "This looks
wrong" is not a finding. If you cannot pin down the correct answer, the case goes in
`bugs/questions/`.

## Setup

The compiler shells out to a vendored QBE and the system `cc`, and links a prebuilt runtime
archive that includes the Boehm garbage collector. `make` stages QBE, the archive, and the bundled
modules in `target/lib/oberon/`, where the compiler looks for them; `cargo build` alone does not.
Build everything and take a baseline:

```
make test
mkdir -p build
```

`make test` panics on the first program that fails. It names that one program and tells you
nothing about the other 127, so a red result on its own does not say whether one thing is broken or
everything is. Find out before you decide anything:

```
for f in tests/corpus/*.Mod tests/failures/*.Mod; do
  m=$(basename "$f" .Mod)
  timeout 60 ./target/debug/oberon -o "build/$m" "$f" >/dev/null 2>&1 || echo "FAIL $m"
done
```

A handful of isolated failures with a mechanism you can explain are findings in their own right.
File them and carry on hunting. Widespread failure means the checkout or the host is wrong, not the
compiler; stop and report that instead, because a broadly red baseline makes every later result
ambiguous.

## How to compile and run one program

```
./target/debug/oberon -o build/Foo path/to/Foo.Mod   # writes build/Foo
./build/Foo                                          # run it
```

Four things matter:

- **Always pass `-o build/<Module>`.** Without it, the compiler writes the executable to the working
  directory, which here is the repository root. The commands in this prompt use paths relative to
  that root.
- **The source file itself can live anywhere.** Its own directory becomes the root for resolving
  `IMPORT`, and the bundled modules are searched second.
- **`build/` is a shared flat directory keyed by module name.** Two test programs named `Foo` in
  different directories overwrite each other's executable. Give every module a distinct name.
- **Run the compiled program in a scratch directory**, not in the repository, if it touches files.

Leave `RUST_BACKTRACE` unset. The driver reports ordinary source diagnostics through the same error
path as internal failures, so turning backtraces on buries the one useful line of every correct
rejection under twenty-five lines of Rust stack — and you will be reading a lot of correct
rejections of your own malformed programs. When you hit a genuine panic, re-run that single command
with `RUST_BACKTRACE=1` to get the stack for the report.

Wrap both the compile and the run in `timeout 60` so a hang shows up as a finding rather than as a
stalled session. Check exit statuses explicitly; a nonzero exit with empty stderr is itself worth a
look.

## Method

Work in small cycles, and keep each cycle honest:

1. Pick a theme from the list below, or invent one.
2. Write a short module — twenty lines is plenty — that exercises one idea. Make it print enough to
   distinguish right from wrong.
3. **Write down the expected output before you run it.** Predicting after the fact is how you talk
   yourself into believing whatever the compiler printed.
4. Compile and run. Compare.
5. On a mismatch, minimize. Strip the program until removing anything else makes the symptom
   disappear. A ten-line reproducer is worth ten times a hundred-line one. When the symptom turns
   on a size — a count, a length, a nesting depth, a number of modules — script the search for the
   threshold rather than guessing at it, and report the exact value where the behavior changes.
   Then vary a second dimension to find out what the threshold is really counting.
6. Run the minimized case twice. Nondeterminism changes the report and points somewhere else in the
   compiler.
7. Confirm your program is actually legal Oberon-07 before filing. Most apparent bugs are invalid
   test programs.
8. Write the finding out, then move on. Do not investigate the compiler source deeply — one line
   naming the file you suspect is enough, and only when it is obvious.

Two techniques give you an oracle without needing an external compiler:

- **Constant versus runtime.** Compute the same expression twice: once folded into a `CONST` or a
  literal expression, once through variables the compiler cannot fold. The two must agree. This
  finds constant-folding bugs with no outside reference at all.
- **Self-checking programs.** Use `ASSERT` and round trips that must hold by definition:
  `FLOOR(FLT(i)) = i`, `CHR(ORD(c)) = c`, `x = (x DIV y) * y + (x MOD y)`, a value written to a file
  and read back, a string inserted then extracted. The program becomes its own oracle.

Read `tests/corpus/` before you pick your first theme, and `bugs/scratch/` from previous runs
alongside it. There are 143 corpus files, and together with the scratch programs they are the real
map of where this compiler has already been checked. Be aware that the corpus already applies the
constant-versus-runtime technique systematically — `DivMod.Mod` and `Builtins.Mod` compute every
arithmetic case twice and print both columns — so on that ground you are not bringing an oracle it
lacks. Ground a passing test already holds is unlikely to be where the bugs are. Aim at the edges
of it, and at what it does not touch at all.

## Where to look

Seed themes, ordered by how thin the existing coverage is rather than by how often compilers in
general get them wrong. Do not treat this as a checklist to march through — follow whatever starts
smelling, and let `tests/corpus/` override this ordering wherever it disagrees.

Two calibration notes from the last run. Backend stress, first below, is the least-tested ground in
the repository and produced both findings; nothing in `tests/corpus/` exercises it. The semantic
themes further down — integers, reals, strings, sets, arrays, records, pointers, procedures,
statements, modules — are the most thoroughly covered, and ten programs aimed across them found
nothing at all.

**Backend stress.** Very deep expression nesting. Hundreds of local variables. Large record and
array copies. Very long identifiers, including ones made long by the module name or the nesting
path rather than by the source name itself. Large string literals. A build with many modules.
Anything that might overflow a frame, a temporary counter, a symbol-length limit, or a static data
limit. Watch for the emitted assembly being valid on one object format and not another.

**The bundled library.** `In` on malformed input and the state of `In.Done` afterwards is the
largest untested gap in the whole project; it needs `.stdin` fixture files, which is why previous
runs have skipped it. Also `Out.Int` with widths narrower and wider than the number, `Out.Real`
formatting, `Strings` operations at and past the ends of their arguments, and `Files` round trips
including reading past the end.

**Integer edge values.** `-2147483648` and `2147483647` as literals, in constant expressions, and
as computed values. `ABS` of the most negative integer. Constant expressions that overflow during
folding. `DIV` and `MOD` with negative operands — Oberon-07 floors, so `(-7) DIV 2` is `-4` and
`(-7) MOD 2` is `1`. `LSL`, `ASR`, and `ROR` with a shift of 0, 31, 32, more than 32, and negative.
`ODD` on negative values. Division by a zero that is only zero at runtime.

**Reals.** Binary32 rounding where a value is not representable. `FLOOR` on negatives, on halves,
and on values too large for an integer. `FLT` on the integer extremes. `PACK` and `UNPACK`.
Division by `0.0`. Real literals with scale factors, and the fact that Oberon-07 requires a decimal
point. `Math` procedures at their domain edges — `sqrt` of a negative, `ln` of zero, `exp` of
something large.

**Characters and strings.** A string literal assigned to an array of exactly the right length, and
one character too short. The empty string. A one-character string where a `CHAR` is expected. Hex
character literals like `0AX`. String comparison with `<` and `>`. An array of `CHAR` with no `0X`
terminator passed where a string is expected. Strings crossing module boundaries.

**Sets.** Element 31. `{31}` and `{0..31}`. A range whose lower bound exceeds its upper bound.
`INCL` and `EXCL` at the boundary. `IN` with an element outside the representable range. Set
constants folded at compile time versus built at runtime. Set difference and symmetric difference.

**Arrays.** `LEN` on each dimension of a multidimensional array, fixed and open. Indexing at
exactly the bound and one past it. Constant indices out of range, which should be caught at compile
time. Whole-array assignment. Open array parameters of two dimensions. Arrays of records, arrays of
pointers, arrays of procedures.

**Records and extension.** Assigning an extended record to a base variable. Type guards `v(T)` that
fail at runtime. `IS` on a `VAR` parameter versus on a pointer. Three or more levels of extension.
The `CASE` type-guard form. Descriptors for records reached through several modules.

**Pointers and the collector.** `NEW`, dereference of `NIL`, pointer equality between a base and an
extension, cyclic structures, and retention — allocate enough to force collections and check that
live data survives. Records containing both pointers and non-pointers. Forward-referenced pointer
bases inside one `TYPE` section.

**Procedures.** Procedure values assigned, passed, returned, compared, stored in arrays and record
fields. `VAR` parameters of procedure type. Calling through a variable that the argument list
changes mid-call. Nested procedures reaching two or more levels out to enclosing locals.
Left-to-right evaluation order of arguments. Deep recursion. Many parameters. A missing `RETURN` on
some path.

**Statements.** `FOR` with a negative `BY`, with a limit expression that has side effects, and with
a limit that is evaluated once rather than each iteration. `CASE` with ranges, with `CHAR` labels,
and with no matching label at runtime. The `ELSIF` form of `WHILE`, which Oberon-07 has and most
people forget. `REPEAT`. Deeply nested control flow.

**Modules.** Initialization order across a diamond of imports. A user module shadowing a bundled
one. Assigning to a variable imported from another module, which is read-only. Exported types whose
fields are not exported. Import cycles. A module whose name collides with a bundled module's.

## Where findings go

Findings live in `bugs/` at the repository root, one directory per finding. Earlier runs may have
left content there already; add to it and continue the numbering rather than starting over.

```
bugs/
  001-neg-div-constant-fold/
    NOTES.md
    NegDivFold.Mod          # the minimized reproducer
    <extra .Mod files>      # only if the bug needs several modules
    <name>.stdin            # only if the program reads stdin
  002-.../
  questions/
    <same shape, for cases you could not resolve>
  scratch/
    <every program that ran and matched its prediction>
  INDEX.md
```

Number directories in the order you find them and never renumber. `INDEX.md` is one line per
finding: number, one-sentence title, category, confidence. Keep it current as you go, so an
interrupted run still leaves something useful behind.

A program that matched its prediction goes in `bugs/scratch/` and stays there. Those are the
coverage record. They are what lets the next run skip ground you already walked, and they are the
evidence behind the part of your report that says a theme turned up nothing.

A failure in the existing suite is a finding like any other and gets its own numbered directory.
Copy the failing test into it under a fresh module name rather than pointing the reproducer at
`tests/`, so the case survives someone changing the test.

`NOTES.md` for each finding:

```markdown
# <one-line title>

Category: wrong answer | compiler crash | toolchain failure | wrong rejection | runtime check |
          generated program crash
Confidence: high | medium | low

## Reproduce

    ./target/debug/oberon -o build/NegDivFold bugs/001-neg-div-constant-fold/NegDivFold.Mod
    ./build/NegDivFold

## Expected

<what should happen, and why — cite the Report section, the grammar production, or
docs/standard-library.md. If you are reasoning from a definition rather than a quoted document,
say so.>

## Actual

<exact output, exit status, and stderr. Paste it; do not paraphrase it.>

## Notes

<optional: what you tried while minimizing, what the symptom is sensitive to, and at most one line
naming the compiler source file you suspect — only if it is obvious.>
```

Write in the present tense, describing the bug as it is. Do not narrate your investigation.

## Rules

- Do not modify anything in `src/`, `lib/`, `runtime/`, or `tests/`. You are reporting, not fixing.
- Do not add test cases to `tests/`. Findings live in `bugs/` until a human triages them.
- Do not commit, stage, or push. `bugs/` stays untracked.
- Keep scratch files inside `bugs/scratch/`. Do not leave stray `.Mod` files around the repository.
  `build/` will accumulate artifacts; that is fine, it is gitignored.
- If you find yourself reading compiler source for more than a few minutes, stop and get back to
  writing programs. Depth is the human's job in triage; breadth is yours.

## Finishing

Stop after you have tried roughly 60 to 80 distinct programs, unless you were given a different
number. Do not treat a finding count as a target or as an early exit. This compiler is mature
enough that a long, careful run can legitimately end with very few findings: the last run wrote ten
programs and filed two, and eight of those ten matched their predicted output exactly. A run that
ends with one solid finding and a clear account of what it ruled out is a good run.

Then report back with:

- How many programs you wrote and how many themes you covered.
- The findings, most severe first, one line each.
- The themes you touched that turned up nothing — that is real information about where the compiler
  is solid.
- Anything you wanted to test but could not, and why.
