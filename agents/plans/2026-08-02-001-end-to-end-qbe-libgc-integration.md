# Slice 1: End-to-end QBE + libgc integration

## Context

The oberon repo is a blank Rust binary crate (hello-world `main.rs`, no deps). Before writing any lexer/parser, this slice proves the *back half* of the pipeline works end to end: a Rust driver emits hardcoded QBE IL, shells out to `qbe` to get assembly, and invokes `cc` to assemble and link it against a tiny C runtime and BDWGC — producing a native executable whose exit code we can verify.

Toolchain confirmed on this machine: `/usr/bin/qbe` (amd64_sysv default), `/usr/bin/cc` (gcc), `/usr/lib/libgc.so`, `/usr/include/gc.h`.

The verifiable behavior: the generated program calls `oberon_init` (GC_INIT), allocates 8 bytes via `oberon_alloc` (GC_MALLOC), stores 42 through the pointer, loads it back, and returns it — so `./build/out; echo $?` printing `42` proves QBE codegen, linking, and a live GC allocation all work.

## Files

### `runtime/oberon.c` (new)

The minimal C runtime — just the allocator seam named in AGENTS.md plus init:

```c
#include <gc.h>

void oberon_init(void)              { GC_INIT(); }
void *oberon_alloc(size_t n)        { return GC_MALLOC(n); }
void *oberon_alloc_atomic(size_t n) { return GC_MALLOC_ATOMIC(n); }
```

No header yet — nothing else consumes it. Compiled fresh by `cc` in the link step (it's three lines; no build caching needed).

### `src/driver.rs` (new)

The driver seam from AGENTS.md. Consts at the top: `QBE: &str = "qbe"`, `CC: &str = "cc"`, `RUNTIME_C: &str = "runtime/oberon.c"`, `BUILD_DIR: &str = "build"`.

`pub fn build() -> anyhow::Result<()>`:

1. Create `build/`, write the hardcoded IL to `build/out.ssa`:

   ```
   export function w $main() {
   @start
           call $oberon_init()
           %p =l call $oberon_alloc(l 8)
           storew 42, %p
           %v =w loadw %p
           ret %v
   }
   ```

2. Run `qbe -o build/out.s build/out.ssa`.
3. Run `cc build/out.s runtime/oberon.c -lgc -o build/out` (cc assembles + links in one step).

Each `Command` gets `.context("running qbe")` etc. via anyhow; a nonzero exit status from qbe/cc becomes an error (`bail!` with the tool name). Log each exact command line with `tracing::debug!` per AGENTS.md.

### `src/main.rs` (replace)

Thin: install a `tracing_subscriber` fmt subscriber on stderr with `EnvFilter` (so `RUST_LOG=debug cargo run` shows the command lines), call `driver::build()`, return `anyhow::Result<()>`.

### `Cargo.toml`

Add deps: `anyhow`, `tracing`, `tracing-subscriber` (with `env-filter` feature). These are the three sanctioned by AGENTS.md.

### `.gitignore`

Add `/build`.

## Verification

```sh
cargo run                 # compiles IL → asm → links
./build/out; echo $?      # expect: 42
RUST_LOG=debug cargo run  # shows qbe/cc command lines on stderr
```

Also sanity-check the intermediate artifacts exist and are inspectable: `build/out.ssa`, `build/out.s`.

## Out of scope (deliberately)

- No CLI args, no source-file input — the IL is a hardcoded string until the lexer/parser exist.
- No `--emit-il` flag yet (leave a `// TODO`).
- No runtime header, no error taxonomy, no build caching.
