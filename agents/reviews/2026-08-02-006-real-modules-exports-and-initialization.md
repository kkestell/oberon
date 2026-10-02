# Review: Real modules, exports, and initialization

Reviewed against the uncommitted working tree on top of commit `4910526`
(`scalar control and predefined operations`). The implementation follows
[the Slice 7 plan](../plans/2026-08-02-007-real-modules-exports-and-initialization.md).

The working tree also changes `AGENTS.md`. That policy change is outside the
Slice 7 plan and was not part of this compiler review.

## Verdict

The module graph, public interfaces, imported-variable protection, aggregate
IR, and initializer order are implemented correctly. The successful and error
corpora cover the important language rules and graph shapes from the plan.

One medium-priority finding was confirmed and fixed during review. No findings
remain.

## Finding

### The root source filename check ignores the extension

The plan and architecture require every source module to live in a file named
exactly `ModuleName.Mod`. Dependencies meet that rule because `lookup` constructs
that exact filename. The explicit root takes a different path.

`build` derives the expected module name with `Path::file_stem`. `Build::compile`
then compares the parsed module name with that stem. Neither function compares
the complete filename with `ModuleName.Mod`.

As a result, `ReviewExtension.txt` containing this valid module was accepted:

```oberon
MODULE ReviewExtension;
END ReviewExtension.
```

This command exited with status zero and wrote no output:

```text
cargo run --quiet -- build/ReviewExtension.txt
```

It produced `build/ReviewExtension.ssa`, `build/ReviewExtension.s`, and the
native executable `build/ReviewExtension`. Running that executable also exited
with status zero and wrote no output. The temporary source and generated files
were removed after the check.

The driver should compare the explicit source's complete filename with
`format!("{}.Mod", module.name)` after parsing. A regression should pass a root
with the right stem and a wrong or missing extension. The expected result is a
source diagnostic at the module name, not a linked executable.

This is limited to the explicit root. Imported dependencies already come from
the exact filename constructed by `lookup`.

### Resolution

`Build::compile` now compares the complete source filename with
`ModuleName.Mod`. The `RootWrongExtension.txt` regression invokes the compiler
with a matching stem and the wrong extension. It checks both the diagnostic and
the failing exit status.

## Verified implementation

The parser now retains export marks instead of rejecting them. It also keeps
separate positions for an import qualifier and its real module name. The cycle
test confirms that an aliased closing edge points at the real name after `:=`.

Semantic analysis returns one IR module and one owned public interface. The
interface contains only marked constants, variables, and procedures. A
variable becomes read-only only when its interface member is installed in a
client. The defining module keeps its writable symbol.

`ApiSupport` mixes marked and unmarked variables in one declaration. Its body
initializes the public variable, and an exported procedure later changes it.
The client reads the same value through two aliases. It also folds an imported
constant and calls exported proper and function procedures with value and
`VAR` parameters.

The read-only error module exercises assignment, a writable `VAR` actual,
`INC`, and `DEC`. It produced these four diagnostics:

```text
tests/errors/modules/imported-write/ImportedWrite.Mod:4:3: cannot assign to 'Support.count': it is read-only
tests/errors/modules/imported-write/ImportedWrite.Mod:5:17: argument 1 is read-only
tests/errors/modules/imported-write/ImportedWrite.Mod:6:7: argument 1 is read-only
tests/errors/modules/imported-write/ImportedWrite.Mod:7:7: argument 1 is read-only
Error: 4 error(s)
```

The driver follows imports depth first in source order. It caches completed
interfaces only for the current invocation. A repeated module name therefore
produces one IR module, while distinct qualifiers still receive their own
module symbols.

The cycle calculation removes an acyclic root prefix. The checked three-module
graph reported the closing edge in `B.Mod` with this exact output:

```text
tests/errors/modules/import-cycle/B.Mod:2:19: import cycle: A -> B -> A
Error: 1 error(s)
```

A dependency whose heading disagrees with its lookup name reports the
dependency path. The checked program produced:

```text
tests/errors/modules/dependency-name-mismatch/Dep.Mod:1:8: module 'NotDep' must be stored in a file named 'NotDep.Mod'
Error: 1 error(s)
```

Source lookup prefers the root directory over the bundled directory. It falls
back to the native `Out` interface only after both source locations fail. The
source `Out` shadow and the aliased native `Out` program both compiled and ran
successfully.

QBE emission combines all source modules into one unit. `Diamond.ssa` defines
one initializer each for `Leaf`, `Left`, `Right`, and `Diamond`. Its `main`
calls them once in that order after `oberon_init`.

The diamond program exited with status zero, wrote no standard error, and
printed:

```text
1234
```

The API program exited with status zero, wrote no standard error, and printed:

```text
10  20
1
1   1
5   5
10
3   1
```

The native `Out` alias program printed `7` and a newline. The source `Out`
shadow program produced no output and exited successfully.

The corpus walker now supports nested module graphs. A `.Mod` file becomes a
test root only when it has a sibling `.expected` file. Successful compilation
and execution now both require a zero status and empty standard error.

## Declined changes

The review does not request per-module object files. One QBE unit is the
simplest implementation of the current whole-program build.

The review does not request initialized flags inside module initializers. The
generated entry point is the only caller, and the completed-module map already
builds one dependency-first call list.

The review does not request a separate diagnostic for a private member. A
public-only interface correctly makes private and absent members equally
unavailable to a client.

The review does not request a persistent interface cache or configurable search
path. Both remain deliberate exclusions in the plan.

## Verification

`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test`,
and `git diff --check` pass. The test run contains fourteen unit tests and two
integration tests.

The four positive Slice 7 roots were compiled and run directly. Each exited
with status zero and wrote no standard error. Their standard output matched the
checked-in expected files.

`build/Api.ssa` loads `ApiSupport.count` through both aliases and calls the same
`ApiSupport` procedure symbols. Its `main` calls `ApiSupport` before `Api`.

The unchanged `Params` module still produces SHA-256
`bc6627e4199e252930401829769ea56477ae76987738f442f8111d8121233291`.
That is the hash recorded before Slice 7.

The old semantic special case for the string `Out` is gone. The old parser
diagnostic for unsupported export marks is also gone. No serialized interface
artifact was written under `build/`.
