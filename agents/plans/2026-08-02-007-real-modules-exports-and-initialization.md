# Slice: Real modules, exports, and initialization

## Context

After Slice 6 the compiler handles the scalar INTEGER and BOOLEAN language in
one source module. The parser already records imports and import aliases, but
semantic analysis recognizes only `Out`. An export mark still stops parsing.
The driver reads, analyzes, and emits only the root source file. The generated
`main` calls only that module's initializer.

This slice replaces that path with a real module graph. A build starts from one
root source file. It finds every imported module, analyzes dependencies before
their clients, emits all source modules into one QBE unit, and links one native
executable. It also introduces a small public interface for each analyzed
module.

Report sections implemented: section 4 for export marks, qualified identifiers,
and the scope of exported declarations; section 8.1 for module qualification in
designators; section 9.1 for the ban on assignment to imported variables; and
section 11 for imports, aliases, exported objects, and module bodies. Section
10.1 also applies because an imported variable cannot be supplied to a writable
`VAR` parameter.

The Report says that a module body runs when the module is loaded. It does not
define file lookup, dependency traversal order, or the mechanism that prevents
repeated initialization. Those are implementation choices and are fixed below.

## Language and build decisions

### Export marks are legal only in the module scope

Report section 4 permits an export mark on an identifier declared in a module's
scope. The `identdef` grammar production also occurs in local declarations, so
the parser must accept the syntax there. Semantic analysis then rejects the
mark when the declaration belongs to a procedure.

This matches Project Oberon's `ORP.CheckExport`. It consumes the mark at every
`identdef`, then reports an error when the current declaration level is not
zero.

This slice supports marks on module constants, individual variable names, and
module-level procedures. One variable declaration may therefore mix public and
private names. A nested procedure cannot be exported. Formal parameter names do
not use `identdef`, so a mark on a parameter remains a parse error.

The old `ExportMark` error module becomes a successful corpus module. New error
coverage pins marks on local constants, local variables, and nested procedures.

### A module interface contains only exported objects

Successful analysis returns the module's IR and a public interface. The first
interface needs only three member kinds:

- A constant carries its folded INTEGER or BOOLEAN value.
- A variable carries its type and final QBE data symbol.
- A procedure carries its parameter modes, parameter types, optional result
  type, and final QBE function symbol.

The interface owns its data and can be cloned freely. It is an in-memory build
result, not a serialized symbol file. Slice 10 will extend it with named types
when `TYPE` declarations arrive.

An interface excludes every unmarked declaration. A client that selects an
unmarked or unknown member therefore receives the existing module-member
diagnostic. The implementation does not keep a second public table merely to
distinguish "private" from "not declared" in that diagnostic.

Declined: exposing the analyzer's complete module scope and checking an export
flag during every qualified lookup. A public-only interface represents the
language rule directly. It also prevents a later lookup path from accidentally
forgetting the visibility check.

### Import aliases declare qualifiers

For `IMPORT X := M`, `X` is the identifier declared in the client's module
scope. `M` is the source module and interface being selected. All interface
members retain symbols based on `M`, so an alias never changes linkage names.

Report section 4 forbids two objects with one identifier in a scope. The
compiler therefore rejects both of these cases:

```oberon
IMPORT M, M;
IMPORT X := M, X := N;
```

Importing one module under different qualifiers is legal:

```oberon
IMPORT X := M, Y := M;
```

The Report does not forbid that form. Project Oberon and oberonc both accept
it. The dependency graph still contains one node for `M`, and its initializer
still runs once.

An import qualifier can shadow a predefined identifier because imports live in
the module scope and predefined identifiers live in the universe scope. A
later declaration with the same name as an import is an ordinary duplicate in
the module scope.

### Imported variables are read-only only in the client

Report sections 9.1 and 11 say that imported variables are read-only. The
declaring module still writes its own exported variable normally. Conversion
from an interface member to a client symbol sets the existing `read_only` flag.

The existing write checks then cover assignment, use as a `VAR` actual, `INC`,
and `DEC`. Reading the variable and passing its value to a value parameter
remain legal. A `FOR` control variable is a bare identifier in the Report
grammar, so a qualified imported variable cannot occur in that position.

Imported constants fold like local constants. Imported procedures use the same
direct-call lowering as local module procedures. No new IR instruction or
calling convention is needed for either case.

### Source lookup is flat and deterministic

The root source file fixes one root directory for the entire build. An import
of module `M` searches these locations in order:

1. `M.Mod` in the root source file's directory.
2. `M.Mod` in the repository's bundled `lib/` directory.
3. The temporary native fallback, which recognizes only `Out` in this slice.

The first regular source file wins. The spelling is case-sensitive. The search
does not use the current importing file's directory, environment variables, or
recursive directory scans. A bundled module therefore resolves its own imports
through the same root-first rule as every other module.

For this compiler's current repository-local execution model, bundled `lib/`
is the literal path relative to the compiler process's working directory. This
matches the existing relative paths for `runtime/oberon.c` and `build/`. The
corpus already fixes that working directory to the repository root.

The root directory takes precedence so an application can supply a module with
the same name as a bundled module. Source lookup also precedes the native
fallback. A user `Out.Mod` therefore replaces the temporary runtime-backed
`Out` for that build. When Slice 17 adds `lib/Out.Mod`, normal lookup will
select it and the native fallback can be removed.

The driver requires every source module to be stored in a file named exactly
`ModuleName.Mod`. This check applies to the explicit root path as well as to
dependencies. The module name after `END` is still checked separately by the
parser.

Missing source modules and name mismatches are source diagnostics. A failure to
read a file that was found remains driver plumbing and uses `anyhow` context.

Declined: include paths, package names, environment configuration, and a search
relative to every importing file. Two fixed directories are enough for the
current compiler and give each module name one identity within a build. A
relocatable installed-library path belongs with packaging work, which does not
exist yet.

### The graph is rebuilt for every compiler invocation

The driver walks imports depth first in their written order. It keeps an active
module-name stack for cycle detection. It also keeps a completed map so a
diamond dependency is parsed, analyzed, and emitted once during that build.

An edge to a name already on the active stack is an import cycle. The diagnostic
is attached to the real module name in that import. Its text starts at the first
occurrence of the repeated name, follows the active stack to its end, and then
repeats the name. Any root prefix that leads into the cycle is excluded. A
direct self-import is the two-occurrence form of the same error.

Each dependency is complete before its client is analyzed. Its interface is
therefore available when the client installs import qualifiers. A semantic
error in a dependency is reported with the dependency's path, and the client is
not analyzed against a partial interface.

The completed map lives only for one invocation. The next invocation rereads,
relexes, reparses, and reanalyzes every required source. There is no timestamp
logic and no on-disk interface cache.

Declined: compiling a client from a cached interface while separately deciding
whether a dependency object is current. The build is small enough that this
would add invalidation behavior without a useful speed gain.

### Initializers use one dependency-first list

Every source module still lowers its body to a procedure named
`$.Module.init`. The graph's completed source modules are already in a stable
dependency-first order. That order becomes the module order in the aggregate
IR.

The generated `main` calls `oberon_init` first. It then calls every source
initializer once in aggregate-IR order. The root initializer is last because
all other source modules are its dependencies.

Depth-first traversal follows each source import list from left to right. This
defines the order between dependencies that do not depend on each other. A
module reached again through another branch is skipped because it is already
complete.

This produces the following shape for a diamond whose root imports `Left`
before `Right`, and where both import `Leaf`:

```text
call $oberon_init()
call $.Leaf.init()
call $.Left.init()
call $.Right.init()
call $.Root.init()
```

The temporary native `Out` interface has no Oberon source body and contributes
no source initializer. Its runtime functions are already linked from
`runtime/oberon.c`, and `oberon_init` runs before any generated module body.

Declined: making every initializer call its direct dependencies and guarding
itself with a global initialized flag. OBNC uses that scheme because it compiles
modules separately. This compiler knows the complete static graph and emits one
entry point, so the flattened call list provides the same dependency order and
exactly-once behavior with no new data or branches.

### All source modules share one QBE unit

The typed IR gains an aggregate program containing source modules in
initialization order. Each IR module owns the globals and procedures that the
current single-module program owns. Semantic analysis still produces one IR
module at a time.

QBE emission writes every module's data and procedures into
`build/RootName.ssa`. It writes one `main` after them. The existing QBE and C
commands still produce one assembly file and one executable.

Oberon export marks affect semantic visibility only. All generated definitions
can remain private QBE definitions because calls between source modules occur
inside one QBE unit. Runtime-backed `Out` procedures remain external C symbols.

Declined: one QBE file and object file per source module. Separate objects would
require backend export decisions and artifact invalidation. Neither is needed
before separate compilation or interface caching exists.

## What remains unsupported after this slice

- `TYPE` declarations and exported named types remain unsupported until Slice
  10. The interface deliberately carries only the scalar declarations that can
  exist now.
- `SET`, `REAL`, arrays, CHAR, BYTE, records, pointers, open arrays, and procedure
  types remain in their later roadmap slices.
- Export marks on record fields remain unavailable because record declarations
  do not exist yet.
- The portable library remains limited to the temporary runtime-backed `Out`
  interface. `In`, `Math`, `Strings`, `Files`, and the Oberon implementation of
  `Out` arrive in Slice 17.
- The optional `SYSTEM` module remains outside the core roadmap.
- There are no configurable search paths, packages, serialized interfaces,
  separate compilation, or dynamic module loading.
- The executable runs the root module body and then exits. It does not provide
  an external command mechanism for activating exported parameterless
  procedures after initialization.

The known INTEGER overflow boundaries recorded by earlier slices remain
unchanged. This slice introduces no new arithmetic or dynamic runtime check.

## Changes by file

### src/ast.rs

Add one small identifier-definition representation containing a name, source
position, and export flag. Use it for constant names, each name in a variable
declaration, and procedure names. Keep formal parameter names unchanged because
their grammar uses plain identifiers.

Replace `Import`'s single position with explicit real-module and qualifier
positions. For `IMPORT X := M`, lookup diagnostics point at `M`, while duplicate
qualifier diagnostics point at `X`. An unaliased import stores the same name and
position in both roles.

### src/parser.rs

Make `identdef` consume and retain an optional export mark. Remove the "not yet
supported: export marks" path. Populate the new AST fields at module and local
declaration sites.

Retain the position of the identifier after `:=` in an aliased import instead
of discarding it. Build the explicit real-module and qualifier fields described
above.

The parser continues to accept marks wherever the grammar uses `identdef`.
Semantic analysis decides whether that declaration belongs to the module scope.

### src/sema.rs

Add the owned module-interface and interface-member types. Change `analyze` to
receive the interfaces of resolved imports. Return the analyzed IR module and
its public interface on success.

Replace the `imports` function's `Out` branch with generic interface
installation. Each AST import selects the interface by its real module name and
declares it under its alias or default qualifier. Interface variables become
read-only client symbols at this boundary.

Record successful exported module declarations in the public interface. Reject
an export flag when the declaration is in a procedure scope. Keep private
declarations only in the normal semantic scope.

Move the two runtime procedure descriptions currently built by `out_scope` into
a function that returns the native `Out` interface. The driver's native fallback
uses that function. No semantic lookup tests the string `Out` after this change.

### src/ir.rs

Split the current single-module `Program` shape into an aggregate `Program` and
a per-source `Module`. The aggregate holds modules in initializer order. The
per-source value holds its name, globals, and procedures.

No instruction, type, address, or calling-convention representation changes.

### src/qbe.rs

Emit the globals and procedures of each source module. Emit one `main` that
calls every module initializer in order after `oberon_init`.

Keep the output in one string. No QBE linkage annotation changes are needed.

### src/driver.rs

Add the depth-first graph walk, fixed source lookup, file-name check, active
stack, completed interface map, and dependency-first IR list. The explicit root
path is parsed through the same source routine as dependencies after its path
has been selected.

Keep source lookup as a plain helper taking the fixed root and library
directories plus a module name. Production passes `lib/` as the library
directory. The explicit argument also lets the lookup rule be tested without
changing process state.

Source diagnostics continue to use the existing `Diagnostic`. The driver calls
`report` with the path of the module that owns each diagnostic. There is no new
diagnostic taxonomy.

The final QBE, assembler, and linker steps remain one invocation each. Their
output names continue to use the root module name.

Add focused unit coverage for the source lookup helper in this file. It must
prove root-directory precedence, bundled-directory fallback, exact file
spelling, and a missing module result. Checked-in fixture directories are
enough, so this needs no new crate.

### src/main.rs

No change. The command-line interface remains `oberon <file.Mod>`, and the
driver still owns the complete build.

### agents/architecture.md

Update the pipeline to show root-driven module discovery before aggregate IR
emission. Document the two source directories, the temporary native fallback,
and dependency-first initializer order.

### tests/corpus.rs

Fix the open finding from the Slice 6 review. A successful corpus binary must
exit with status zero, write no standard error, and match standard output byte
for byte.

Teach the fixture walk to recurse and to treat only a `.Mod` file with a sibling
`.expected` file as a test root. A `.Mod` file without that pair is a dependency
source. This permits each module graph to live in its own directory without
turning every support module into a separate test case.

Keep diagnostic paths relative to the repository root. The nested paths then
remain stable in expected files.

### runtime/oberon.c

No change. `oberon_init`, `oberon_out_int`, and `oberon_out_ln` keep their
current ABI.

## New corpus modules

`tests/corpus/modules/api/` contains the public-interface workhorse. Its support
module exports INTEGER and BOOLEAN constants and variables. It also exports
proper and function procedures with value and `VAR` parameters. The client
imports that module under two aliases, which proves that repeated real-module
imports with distinct qualifiers are legal and produce one graph node.

One support declaration contains both a marked and an unmarked variable. The
initializer assigns the marked variable. An exported procedure mutates that
same variable. The client reads it through both aliases before and after the
call. This proves that the declaring module can write it, clients cannot, and
both aliases use one data object.

The API client also folds qualified constants. It reads the exported BOOLEAN
variable after dependency initialization. It calls exported procedures through
both aliases. It passes its own locals to exported `VAR` parameters. Private
support objects are present but unused by the successful client.

`tests/corpus/modules/diamond/` contains `Leaf`, `Left`, `Right`, and a root.
Each source module writes one distinct digit during initialization. The root
imports `Left` before `Right`, and both branches import `Leaf`. The exact output
is `1234` followed by a newline. That output proves dependency-first order,
left-to-right sibling order, and one execution of `Leaf`.

`tests/corpus/modules/out-shadow/` contains a source module named `Out` and a
root that imports it. The source `Out` exports a member absent from the native
interface. The root uses that member and produces no output. Successful
execution proves that root-directory source lookup precedes the native fallback.

`tests/corpus/modules/out-alias/` contains a root with `IMPORT O := Out`. It
writes through `O.Int` and `O.Ln`. This proves that the native fallback is
installed through the same alias path as a source-module interface.

Move the old top-level `ExportMark` program from the error corpus to the
successful corpus with empty expected output. Its top-level exported variable
is now valid.

## New error modules

Each error root has a sibling `.expected` file. Its dependency sources do not.

- `PrivateUse` selects a private constant, variable, and procedure from a
  support module. Its private variable shares one declaration with a marked
  variable. All three private accesses must fail.
- `ImportedWrite` assigns to an exported variable, passes it to a writable
  `VAR` parameter, and calls both `INC` and `DEC` on it. Each applicable write
  path must report that the imported variable is read-only.
- `LocalExport` marks a procedure-local constant, a procedure-local variable,
  and a nested procedure. The parser must accept all three marks, and semantic
  analysis must reject their scope.
- `DuplicateImport` repeats one qualifier and reuses one alias for a different
  real module. It also declares a module-level object with that alias after the
  import list. Duplicate qualifier and declaration diagnostics must point at
  the qualifier and declaration that collide.
- `AliasScope` imports `X := M` and then tries to select a member through `M`.
  The real module name must not be declared as a second qualifier.
- `MissingImport` names a module found in neither source directory nor the
  native fallback.
- `ImportCycle` has a root that leads into a cycle between two dependencies.
  The closing edge uses an alias. The diagnostic must point at the real module
  name after `:=`, omit the acyclic root prefix, and repeat the first cyclic
  name at the end.
- `RootNameMismatch` declares a module whose name does not match its root file.
- `DependencyNameMismatch` imports a correctly named file whose module heading
  declares another name. The diagnostic must name the dependency source path.

No runtime-failure module is added because this slice introduces no dynamic
check.

## Verification

1. Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
   `cargo test`, and `git diff --check`.
2. Run the API, diamond, source-`Out`, and native-`Out` alias gate binaries by
   hand. Confirm status zero and empty standard error before reading their
   expected output.
3. Inspect `build/Diamond.ssa`. Each source initializer must be defined once.
   The `main` call list must contain `Leaf`, `Left`, `Right`, and the root once
   each in that order.
4. Inspect the API client's IL. Imported variable loads must name the support
   module's data symbols. Calls through both aliases must name the same support
   procedure symbols.
5. Compile one unchanged single-module corpus program and compare its IL with
   the pre-slice output. Moving `Out` into the interface path must not otherwise
   change that program's data, procedures, or initializer body.
6. Run every error root directly once. Confirm that errors inside dependencies
   print the dependency path. Confirm that the cycle is reported at the real
   module name on the closing import and excludes the root prefix.
7. Search `src/sema.rs` for the old `import.name == "Out"` branch and search
   `src/parser.rs` for the old export-mark unsupported diagnostic. Both must be
   gone.
8. Touch a dependency during a manual check and rebuild its root. Confirm from
   tracing output that the dependency is read and analyzed again. No persistent
   interface artifact should exist under `build/`.

## Order of work

1. Fix the successful-corpus status and standard-error checks. Add recursive,
   paired-root fixture discovery.
2. Add export flags and the two import positions to the AST and parser. Move the
   old export-mark test to the successful corpus.
3. Add semantic module interfaces, public-interface construction, local-mark
   diagnostics, and generic import installation.
4. Move the native `Out` descriptions behind the same interface type and remove
   the semantic module-name branch.
5. Split aggregate and per-source IR. Update QBE emission for several modules
   and the ordered initializer call list.
6. Add driver source lookup, file-name validation, graph traversal, cycle
   detection, and dependency-first analysis.
7. Add the source lookup tests, successful module graphs, and static-error
   graphs. Add regression modules for any bug found during implementation.
8. Update the architecture document and complete the verification list.
