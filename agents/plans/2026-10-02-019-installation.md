# Installation

## Goal

The compiler only works from the repository root: it reads `runtime/` and `lib/` relative to the working directory, writes into `build/`, runs `qbe` from `PATH`, and links whatever BDWGC the host provides (Homebrew on macOS). After this change, `make install` builds and installs a user-local compiler, and the installed `oberon` compiles and runs a program from any directory. A C compiler is the only host requirement.

## Related code

- `src/driver.rs` — `build` hard-codes `RUNTIME_C`, `STANDARD_C`, `LIB_DIR`, `BUILD_DIR`, `QBE`, and the `HOMEBREW` include and library paths. `lookup` and the bundled-origin check in `build` compare against `LIB_DIR`.
- `src/main.rs` — takes exactly one argument, the source path.
- `runtime/oberon.c`, `runtime/standard.c` — compiled on every build today; `oberon.c` includes `<gc.h>`.
- `tests/corpus.rs` — runs the compiler from the repository root and runs `build/<stem>`.
- `AGENTS.md`, `agents/roadmap.md` — describe QBE and BDWGC as host dependencies and list Installation as a gap.

## Decisions

**Installed layout.** `make install` copies the compiler to `$(PREFIX)/bin/oberon` and everything else to the support directory `$(PREFIX)/lib/oberon/`: the bundled `*.Mod` modules, the `qbe` executable, and `liboberon.a`. `PREFIX` defaults to `$(HOME)/.local`, so no root is needed. Install replaces the whole support directory, so a module removed from `lib/` does not linger.

**Support directory resolution.** The driver finds the support directory at `../lib/oberon` from the canonicalized `std::env::current_exe()`, so a symlinked `oberon` still finds its own install. There is no environment variable and no fallback. A missing support directory fails with an `anyhow` error naming the path it looked for.

**Development uses the same layout.** Cargo puts the compiler at `target/debug/oberon` or `target/release/oberon`, so the support directory resolves to `target/lib/oberon/` for both. `make` populates it, and `make test` runs `cargo test` after populating it. A plain `cargo test` uses whatever `make` last staged, so runtime C and `lib/*.Mod` edits need `make` (or `make test`) to take effect.

**Vendored BDWGC and QBE.** Unmodified release trees go in `vendor/bdwgc/` (latest 8.2 release) and `vendor/qbe/` (1.3 or later, required for Apple ARM64), with versions, source URLs, and licenses recorded in `vendor/README.md`. BDWGC is built as its single translation unit, `vendor/bdwgc/extra/gc.c`, without thread support: generated programs are single-threaded. QBE is built with its own Makefile, which picks the host's default target.

**One prebuilt runtime archive.** `make` compiles `gc.c`, `runtime/oberon.c`, and `runtime/standard.c` with `-O2 -Ivendor/bdwgc/include` and archives them into `liboberon.a`. A program link becomes `cc <asm> <support>/liboberon.a -lm -o <output>`. No headers are installed, and the collector is linked statically, so the program does not depend on a host `libgc`. Recompiling the collector on every build would cost seconds; the archive keeps a build to one QBE run and one link.

**Command line and output.** `oberon [-o output] file.Mod`. Without `-o`, the executable is written to the current directory, named after the source file's stem. The `.ssa` and `.s` intermediates go in a fresh `std::env::temp_dir()/oberon-<pid>` directory, removed after linking whether or not it succeeded. Missing or extra arguments, an unknown flag, or `-o` without a value print usage and exit 2.

## Test plan

- The existing corpus passes under `make test`, with the harness passing `-o build/<stem>` and creating `build/` itself.
- New `tests/corpus.rs` test `compiles_outside_the_checkout`: in a fresh temporary directory, write `Hello.Mod` importing `Out`, run the compiler there on `Hello.Mod` with no `-o`, and check that the directory then holds only `Hello.Mod` and `Hello`, and that `./Hello` prints the expected line. This covers support-directory resolution, bundled lookup independent of the working directory, the default output, and intermediate cleanup.
- The `lookup` unit tests stay unchanged.
- Manual: `make install PREFIX=$(mktemp -d)`, then from `/tmp` compile and run a program importing `Out` and `Files` with `$PREFIX/bin/oberon`; `otool -L` (macOS) or `ldd` (Linux) on the program shows no `libgc`. Check `oberon`, `oberon -o`, and `oberon a.Mod b.Mod` print usage and exit 2.

## Implementation plan

1. Add `vendor/bdwgc/`, `vendor/qbe/`, and `vendor/README.md`. Add QBE's in-tree build outputs (`vendor/qbe/qbe`, `vendor/qbe/config.h`, object files) to `.gitignore`.
2. Add `Makefile` with `PREFIX ?= $(HOME)/.local` and `SUPPORT = target/lib/oberon`:
   - `all` (default): `cargo build --release`, plus `$(SUPPORT)/qbe`, `$(SUPPORT)/liboberon.a`, and a copy of each `lib/*.Mod` in `$(SUPPORT)`, each rebuilt when its sources change. Objects go in `target/obj/`.
   - `test`: the `$(SUPPORT)` targets, then `cargo test`.
   - `install`: `all`, then install `target/release/oberon` into `$(PREFIX)/bin` and replace `$(PREFIX)/lib/oberon` with a copy of `$(SUPPORT)`.
   - `uninstall`: remove `$(PREFIX)/bin/oberon` and `$(PREFIX)/lib/oberon`.
   - `clean`: `cargo clean` and the QBE build outputs.
3. `src/driver.rs`: add `support_dir()` and change `build` to `build(source: &Path, output: &Path)`. Replace `RUNTIME_C`, `STANDARD_C`, `LIB_DIR`, `BUILD_DIR`, and `HOMEBREW` with `SUPPORT_DIR = "lib/oberon"` and `RUNTIME_ARCHIVE = "liboberon.a"`; run QBE from the support directory; use the support directory as `lib_dir` and in the bundled-origin check; write intermediates to the temporary directory and link to `output`.
4. `src/main.rs`: parse `-o` and the source path, default the output to the source file's stem in the current directory, and pass both to `driver::build`. Update the usage line.
5. `tests/corpus.rs`: create `build/`, pass `-o build/<stem>` in `compile` and in `generated_module`, update the header comment, and add `compiles_outside_the_checkout`.

## Documentation updates

- `AGENTS.md`: under Architecture, say QBE and BDWGC are vendored in `vendor/` and replace "Apple ARM64 needs QBE 1.3 or later" accordingly; under Runtime, say the runtime and collector are prebuilt into `liboberon.a` in the support directory. Add a short Building section: `make`, `make test`, `make install [PREFIX=...]`, and that a plain `cargo test` uses what `make` last staged.
- `agents/roadmap.md`: remove the Installation item.
