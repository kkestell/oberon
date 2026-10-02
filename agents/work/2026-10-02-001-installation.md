# Installation

## Plan

`agents/plans/2026-10-02-019-installation.md`

## Summary

`make install` builds and installs `bin/oberon` and `lib/oberon/` (bundled modules, vendored QBE 1.3, and `liboberon.a` with BDWGC 8.2.12 and the runtime) under `PREFIX`, by default `~/.local`. The installed compiler compiles and runs programs from any directory with only the system C compiler on `PATH`. The goal is met on macOS ARM64; Linux was not tried.

## Departures from the plan

- The collector is compiled with `-DNO_EXECUTE_PERMISSION`. Without it, BDWGC maps its heap executable and every program on Apple ARM64 aborts with "Cannot allocate executable pages". BDWGC's configure and `Makefile.direct` builds define it by default; the single-file build does not.
- `same_path` now compares canonical paths, and the root bundled-origin check in `build` uses it. The support directory comes from the canonicalized executable, so comparing it with an uncanonicalized source path would misclassify a bundled file reached through a symlink (macOS `/var` vs `/private/var`, for one).

## Decisions

- `-o` and the source may appear in either order; a repeated `-o`, a second source, or any other argument starting with `-` is a usage error.
- Intermediates go in `temp_dir()/oberon-<pid>`, removed after linking even when QBE or `cc` fails.

## Checks run

- `make test` — 43 unit tests and 4 corpus tests pass, including `compiles_outside_the_checkout`.
- `cargo fmt` and `cargo clippy --all-targets -- -D warnings` — clean.

## Manual verification

1. Install to a fresh prefix and build a program using `Out` and `Files` from another directory, with Homebrew off `PATH`.

   ```sh
   P=$(mktemp -d)/prefix; make install PREFIX=$P
   cd $(mktemp -d)   # write Demo.Mod importing Out and Files
   PATH=/usr/bin:/bin $P/bin/oberon Demo.Mod && ./Demo && otool -L Demo
   ```

   Only `Demo` and `Demo.Mod` remain in the directory; `./Demo` prints its output; `otool -L` lists only `libSystem`.

2. A symlinked compiler and `-o`: `ln -s $P/bin/oberon oc && ./oc -o d2 Demo.Mod && ./d2` works.

3. Usage errors: `oberon`, `oberon -o`, `oberon a.Mod b.Mod`, and `oberon -x a.Mod` print usage and exit 2. A compiler copied away from its install fails with `cannot find the support directory <path>`.

## Follow-up work

- Object rules do not depend on the `Makefile`, so changing `CFLAGS` needs a manual `rm -r target/obj`.
- A module deleted from `lib/` stays in `target/lib/oberon/` until `cargo clean`; `make install` itself replaces the installed directory.
- Linux is untested; BDWGC on Linux may need further defines that its configure build supplies.
