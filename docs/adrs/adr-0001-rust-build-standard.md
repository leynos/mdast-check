# Architectural decision record (ADR) 001: Adopt the Rust build standard for development builds

## Status

Accepted on 8 October 2026. Development, test, lint and typecheck builds use
the parallel `rustc` frontend (`-Zthreads=8`) and, on Linux, the `mold` linker,
while coverage and release builds stay off the fast flags.

## Date

2026-10-08.

## Context and problem statement

Edit-compile cycles are dominated by frontend time and by link time. The
estate's build standard (`rust-build-defaults`) addresses both. Cargo applies
exactly one `rustflags` source, and an assigned `RUSTFLAGS` replaces every
configuration source, so a recipe or workflow step that assigns `RUSTFLAGS`
silently loses the flags unless it restates them. Measurements and shipped
artefacts also need a baseline that the fast flags would disturb.

## Decision drivers

- Faster local and CI development builds without changing what the code means.
- Coverage numbers that do not depend on the fast flags.
- A release that ships from the platform linker.
- A flag lost through a recipe or workflow edit must fail a test, not pass
  quietly.

## Options considered

- Configure the flags only in `.cargo/config.toml`. This leaves every recipe
  and CI step that assigns `RUSTFLAGS` without them.
- Restate the flags in each recipe and step, held by a contract test. This is
  the option taken.
- Adopt Cranelift for development builds. The developers' guide records why
  this repository does not.

## Decision outcome

`.cargo/config.toml` carries the flags in every `rustflags` source, and adds the
`mold` linker flag to the Linux table only. The Makefile composes the same
flags into each development recipe, keeping the caller's own `RUSTFLAGS`.
Coverage assigns its own flags and ignores the caller's. The release recipe
keeps the caller's flags and names neither fast flag, so it ships from the
platform linker; a direct `cargo build --release` takes the configuration's
flags unless `RUSTFLAGS` is assigned. Cranelift is not adopted; the developers'
guide records the measurement and the reason.

`tests/build_standard_contract.rs` reads the Cargo configuration sources and
the commands `make -n` prints for each target, and the `setup-rust` steps of
the workflows, so a flag lost through a recipe or workflow edit fails there.

## Consequences

- Linux builds require `mold`. The flag reaches the linker through `cc`, which
  must be GCC 12.1 or newer, or clang.
- A change to a recipe or workflow step that assigns `RUSTFLAGS` must restate
  the flags, and the contract says which clause fails when it does not.
- Revisit Cranelift when the toolchain or the failing tests change.

## Addendum (2026-11-06)

Whitaker is the one exception to the flag standard above. It runs on its
installer-managed toolchain, which cannot take the repository's frontend flag
or linker argument, so its recipe assigns the warning policy alone
(`RUSTFLAGS="-D warnings"`), never expands the caller's `RUSTFLAGS`, and clears
`CARGO_ENCODED_RUSTFLAGS`, which Cargo reads in preference to `RUSTFLAGS`. This
changes the earlier decision only for that one command: every other recipe
still restates the standard flags, and the consequence that an assigning recipe
must restate them stands. The contract exempts a direct Whitaker command from
the standard's flags, and the Concordat rule checks its isolation.

The development build footprint decision is recorded in the developers' guide
section "Development build footprint": incremental builds and debug information
are off in the development profile, and `[unstable] embed-metadata = false`
keeps rlibs free of embedded metadata. The trade-off is that debugging needs
`CARGO_PROFILE_DEV_DEBUG=true`.
