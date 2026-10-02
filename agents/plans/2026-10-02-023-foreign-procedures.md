# Foreign procedures

## Goal

A program can call a C function directly. A procedure heading may end in
`IS "symbol"` instead of a body, which declares a *foreign procedure*: a call
to it is a plain C call to that symbol, with every argument in its C form and
no hidden arguments. A C file beside a module, `M.c` next to `M.Mod`, is
compiled and linked into the program, so a library whose types do not cross
the boundary gets a thin shim. The compiler forwards `-I`, `-L`, and `-l`
options to `cc`.

Two demos under `demos/` prove the design: a zlib round trip, and an SDL3
window that draws a rectangle and closes on quit. A corpus test covers the
mechanism with libc and a shim only, so `cargo test` needs no extra library.

```oberon
MODULE Zlib;

PROCEDURE CompressBound*(sourceLen: INTEGER): INTEGER IS "compressBound";
PROCEDURE Crc32*(crc: INTEGER; buf: ARRAY OF BYTE; len: INTEGER): INTEGER IS "crc32";
(* compress returns a C int, so Zlib.c wraps it to return int64_t. *)
PROCEDURE Compress*(VAR dest: ARRAY OF BYTE; VAR destLen: INTEGER;
    source: ARRAY OF BYTE; sourceLen: INTEGER): INTEGER IS "zlib_compress";

END Zlib.
```

The private `OberonRuntime` interface and the bundled modules are unchanged.
Converting them to foreign procedures is a later plan.

## Related code

- `src/lexer.rs` — `IS` is already `Tok::Is`; nothing changes here.
- `src/parser.rs` `proc_declaration` — parses `PROCEDURE ident [params] ;
  declarations [BEGIN ...] [RETURN ...] END ident ;`. The foreign form branches
  after the formal parameters.
- `src/ast.rs` `ProcDecl` — gains the foreign symbol.
- `src/sema/mod.rs` `procedure` (about line 366) — declares the symbol, builds
  the `ProcBuilder`, lowers formals, locals, nested procedures, and the body.
  A foreign procedure stops after declaring and exporting.
- `src/sema/mod.rs` `resolve_procedure_signature` (about line 3553) — rejects
  structured result types; the foreign type restrictions go beside it.
- `src/sema/mod.rs` `lower_call` (about line 2095) and the argument loop that
  follows it — pushes the hidden open-array lengths (about line 2236), the
  string-literal length (about line 2201), and the record descriptor (about
  line 2232). A foreign call pushes only the address or value.
- `src/sema/mod.rs` the `Symbol::Proc` arm of the designator source (about
  line 795) — where a procedure becomes a value, gated by `eligible`.
- `src/sema/symbols.rs` `Symbol::Proc`, `Member::Proc`, `client_symbol` —
  carry the procedure's symbol and type to callers and importing modules.
- `src/ir.rs` `Inst::Call`, `CallTarget::Direct` — already a call to a named
  symbol with explicit arguments.
- `src/qbe.rs` `Inst::Call` emission (about line 401) and `class` — the result
  class comes from `ir::Ty`; `Bool` and `Byte` are `l`, which is wrong for a
  C `bool` or `unsigned char` result.
- `src/driver.rs` `build`, `Build::compile`, `lookup`, `holds`, `assemble` —
  the module graph, exact-spelling file lookup, and the fixed `cc` command.
- `src/main.rs` `arguments` — accepts only `-o` and the source.
- `tests/corpus.rs` — a corpus root is a `.Mod` with a sibling `.expected`;
  other files in the directory are its dependencies.
- `agents/roadmap.md` — the Host access gap this closes.

## Decisions

**Syntax.** `PROCEDURE Name*(params): T IS "symbol";` with no body and no
closing `END Name`. OBC uses this form. `IS` is already a keyword, and the
explicit symbol lets the Oberon name differ from the C name. A foreign
procedure is allowed only at module scope; nested, it reports `a foreign
procedure must be declared at module scope`. The string must be a non-empty
string literal; it is used verbatim as the linker symbol.

**No hidden arguments.** An Oberon call passes an open array as address plus
length, a VAR record as address plus descriptor. A foreign call passes exactly
one register per formal parameter, in declaration order, so the Oberon
signature mirrors the C prototype one for one. zlib's `compress(dest,
&destLen, source, sourceLen)` interleaves lengths and pointers, so an implicit
length can never land in the right place; the caller spells it out. A string
literal passed to an open `ARRAY OF CHAR` formal passes its address only, and
the Report's terminating `0X` makes it a C string.

**Types that may cross.** A foreign parameter is a scalar (`INTEGER`, `REAL`,
`CHAR`, `BYTE`, `BOOLEAN`, `SET`), a VAR scalar, or an array, fixed or open,
value or VAR, whose innermost element is a scalar. A foreign result is a
scalar. Records, pointers, and procedure types are rejected with `a foreign
procedure cannot have the {record|pointer|procedure} {parameter|result} type
{ty}`. A VAR record would need a descriptor C cannot supply. A `POINTER`
crossing to C would hide a heap object from the collector or let C memory,
which has no descriptor header, masquerade as an Oberon object; C-owned
memory crosses as an `INTEGER` handle, which is why `INTEGER` holds an
address. Callbacks are a later change.

| Oberon | C |
| --- | --- |
| `INTEGER`, `SET` | `int64_t`, `long`, `size_t`, any pointer |
| `REAL` | `double` |
| `CHAR`, `BYTE`, `BOOLEAN` by value | `int` and narrower, passed zero-extended in a 64-bit register |
| `VAR x: T` for scalar `T` | `T*` with the 64-bit layout, so `int64_t*`, `double*`, `unsigned char*`, `int32_t*` for `BOOLEAN` |
| `ARRAY OF T`, `ARRAY n OF T`, `VAR` or value | `T*` to the first element |

**Results of C `int`.** `INTEGER` means a 64-bit result. A C function that
returns `int` leaves the upper half of the register undefined on both
`amd64_sysv` and `arm64_apple`, so such a function needs a one-line wrapper
in the sibling `.c` returning `int64_t`. `BOOLEAN`, `CHAR`, and `BYTE`
results are received in the `w` class and zero-extended with `extub`, which
is what a C `bool` or `unsigned char` return needs on both targets. A width
marker in the declaration can come later if shims prove tedious.

**Not a procedure value.** A foreign procedure has no Oberon calling
convention, so an indirect call through an Oberon procedure type would push
hidden lengths. `Symbol::Proc` and `Member::Proc` gain `foreign: bool`, and
the designator source rejects it with `foreign procedure 'Name' cannot be
used as a value`. Importing modules see the same flag through
`client_symbol`.

**Link options.** `-I DIR`, `-L DIR`, and `-l NAME`, attached or separate,
are forwarded to `cc` after `liboberon.a` and `-lm`, so a shim can reference
the library. A binding module cannot name its own library; the program's
build does. The demos' Makefile carries the flags.

**Sibling C file.** When `Build::compile` compiles `dir/M.Mod` and `holds(dir,
"M.c")` is true, `dir/M.c` is added to the `cc` command line beside the
assembly. One `cc` invocation compiles and links everything, so no object
files appear. The bundled modules have no sibling `.c`; `runtime/standard.c`
stays in the archive.

**Demos.** `demos/zlib/` holds `Zlib.Mod` with `Zlib.c` and the program
`Roundtrip.Mod`; `demos/sdl/` holds `SDL.Mod` with `SDL.c` and the program
`HelloSDL.Mod`. A binding module separate from the program shows a foreign
procedure imported across modules and a sibling `.c` found by the imported
module's path. `demos/Makefile` builds each with `pkg-config` for SDL3 and
`-lz` for zlib. Nothing in `cargo test` depends on `demos/`.

## Test plan

- `tests/corpus/Foreign/ForeignCalls.Mod` imports `Host.Mod` from the same
  directory, which declares foreign procedures bound to libc and to
  `Host.c`. Expected output pins: `strlen` of a string literal (address only
  crosses); `sqrt` and `floor` taking and returning `REAL`; `strtod` writing a
  `VAR INTEGER` end pointer, reported as the parsed value and the offset from
  the string's address obtained via a shim; `memset` on a `VAR ARRAY OF BYTE`
  with an explicit length, result assigned to an `INTEGER` and ignored; shim
  functions returning `bool` and `unsigned char` with garbage in the upper
  register bits, to pin the `extub` path; a shim taking `int64_t*` and
  `double*`; a shim wrapping an `int`-returning function; `SET` passed and
  returned as `uint64_t`; a fixed `ARRAY 4 OF INTEGER` passed by value and by
  VAR.
- `tests/errors/ForeignRecordParam.Mod`, `ForeignPointerResult.Mod`,
  `ForeignProcParam.Mod` — the type restriction diagnostics.
- `tests/errors/ForeignNested.Mod` — a foreign procedure inside a procedure.
- `tests/errors/ForeignAsValue.Mod` — assigning a foreign procedure to a
  procedure variable.
- `tests/errors/ForeignEmptySymbol.Mod` — `IS "";`.
- `tests/errors/ForeignWithBody.Mod` — `IS "x"` followed by `BEGIN`; the
  parser expects `;`.
- `tests/corpus.rs` gains a test that compiles with `-l m -L /tmp -I .` to
  show the options are accepted and reach `cc`, and one that passes an
  unknown `-x` option and expects the usage exit.
- `demos/`: `make -C demos` builds both programs; `demos/zlib/Roundtrip`
  prints `round trip ok` and the CRC-32 of its input as a hex constant;
  `demos/sdl/HelloSDL` opens a window titled `Hello, Oberon`, draws a
  rectangle, and exits on the quit event or a key press. Run by hand, not by
  `cargo test`.

## Implementation plan

- `src/ast.rs`: add `foreign: Option<String>` to `ProcDecl`.
- `src/parser.rs` `proc_declaration`: after the formal parameters, if the next
  token is `Tok::Is`, take a string literal, require `;`, and return a
  `ProcDecl` with `foreign` set and empty declarations and body. Otherwise
  the existing path. An empty string literal is a parse error at the string's
  position.
- `src/sema/symbols.rs`: add `foreign: bool` to `Symbol::Proc` and
  `Member::Proc`; `client_symbol` copies it and sets `eligible: !foreign`.
  Set `foreign: false` in `runtime_interface`.
- `src/sema/mod.rs` `procedure`: when `foreign` is set, require module scope,
  resolve the signature, check the foreign type restrictions on every formal
  and the result, declare `Symbol::Proc { symbol: <the C symbol>, foreign:
  true, eligible: false }`, export `Member::Proc`, and return without a
  `ProcBuilder`. Nothing is emitted for a foreign procedure.
- `src/sema/mod.rs` `resolve_procedure_signature`: add a `foreign: bool`
  parameter, or a separate check over the resolved formals, that reports the
  restricted types.
- `src/sema/mod.rs` designator source: before the `eligible` check, a
  `foreign` `Symbol::Proc` reports `foreign procedure 'Name' cannot be used
  as a value`.
- `src/sema/mod.rs` `lower_call` and the argument loop: carry `foreign` from
  the `Symbol::Proc`; when set, skip the string-literal length, the open-array
  lengths, and the descriptor, and set `foreign` on the emitted `Inst::Call`.
- `src/ir.rs`: add `foreign: bool` to `Inst::Call`. All existing constructors
  set `false`.
- `src/qbe.rs` `Inst::Call`: when `foreign` and the result `Ty` is `Bool` or
  `Byte`, write the call with `=w` into a fresh temporary and then `extub` it
  into the destination as `l`. Otherwise unchanged.
- `src/main.rs` `arguments`: accept `-I`, `-L`, `-l` with attached or
  separate values and collect them in order as `cc_args: Vec<String>`;
  return them with the source and output. Update the usage line to `usage:
  oberon [-o output] [-I dir] [-L dir] [-l lib] <file.Mod>`.
- `src/driver.rs`: `build` takes `cc_args: &[String]`. `Build` gains
  `shims: Vec<PathBuf>`; `compile` pushes `dir/M.c` when `holds(dir, "M.c")`.
  `assemble` takes the shims and `cc_args` and appends them after the
  assembly and after `liboberon.a -lm` respectively.
- `tests/corpus/Foreign/`: `ForeignCalls.Mod`, `ForeignCalls.expected`,
  `Host.Mod`, `Host.c`.
- `tests/errors/`: the six error cases and their `.expected` files.
- `tests/corpus.rs`: the two option tests.
- `demos/zlib/Zlib.Mod`, `Zlib.c`, `Roundtrip.Mod`: bindings for
  `compressBound`, `crc32`, and wrapped `compress` and `uncompress`;
  the program compresses a fixed text, decompresses it, compares, and prints
  `round trip ok` and the CRC-32 with `Out.Hex`.
- `demos/sdl/SDL.Mod`, `SDL.c`, `HelloSDL.Mod`: bindings for `SDL_Init`,
  `SDL_CreateWindow`, `SDL_CreateRenderer`, `SDL_SetRenderDrawColor`,
  `SDL_RenderClear`, `SDL_RenderPresent`, `SDL_PollEvent`, `SDL_Delay`,
  `SDL_DestroyRenderer`, `SDL_DestroyWindow`, `SDL_Quit`, with constants
  `InitVideo = 20H`, `EventQuit = 100H`, `EventKeyDown = 300H`, and
  `EventSize = 128`. `SDL.c` supplies `sdl_fill_rect(renderer, x, y, w, h)`
  taking doubles and building the `float` `SDL_FRect`. The program reads the
  event type from the first four bytes of a `VAR event: ARRAY EventSize OF
  BYTE` passed to `PollEvent`. On `Init` failure it prints a message and
  exits with status 1 through `Program.Exit`.
- `demos/Makefile`: targets `zlib` and `sdl`, default both; `OBERON ?=
  ../target/release/oberon`; SDL flags from `pkg-config --variable=includedir
  sdl3` and `pkg-config --libs sdl3`; zlib with `-lz`; `clean`.

## Documentation updates

- `docs/foreign-procedures.md` (new): the syntax, the type table, the
  no-hidden-arguments rule, `int` results and shims, the sibling `.c`, the
  options, the collector hazard of handing C a pointer it alone holds, and a
  pointer to `demos/`.
- `AGENTS.md`: in Architecture, after `SYSTEM is not implemented`, state that
  foreign procedures bind C symbols and point to the new doc; in References,
  mention `demos/` beside `examples/`.
- `agents/roadmap.md`: replace the Host access item with what remains: C `int`
  results need a shim, no way to read C-owned memory, no callbacks.
- `docs/about-oberon-07.md`, "What is not there": one sentence noting that
  this compiler adds foreign procedures as its only extension, with a link.
