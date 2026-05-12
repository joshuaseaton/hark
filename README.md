# hark

_hark_ ("**H**ere, **a**nother **R**ust **k**ernel") is an embedded Rust
microkernel. (Or maybe, "**H**ark! **A**nother **R**ust **k**ernel"?)

Intended to be a flexible micro-/uni- kernel for embedded SoCs. Will feature
extensible built-in support for (hopefully) a wide array of RISC-V and Cortex-M
SoCs, and will be straightforward to port to new ones.

**Reality check**: currently we only support QEMU's RISC-V virt machine and the
SiFive HiFive1 Rev B and are non-SMP. We also can't do much beyond booting to a
shell at the moment.

## Features (so far):

- Cargo-native app model: just have you `bin` depend on the hark crate! (well,
  and set a few environment variables to configure the board).
- Backtraces with offline symbolization, courtesy of the LLVM symbolizer markup;
- A shell over serial, and a simple framework to contribute custom shell
  commands (see `#[hark::shell::command]`);
- A testing framework, where additional tests (beyond hark's own) can be
  contributed from anywhere in the app's crate (see `#[hark::test]`);
- Cooperative and pre-emptive multitasking;

TODO: Make hark do more, and say more about it!

## The build system

An aim for this project was to ensure that `cargo build` just worked. Cargo
doesn't make embedded development easy, however, and to support multiple kinds
of targets we were left coming up with a layer above cargo to "set" the desired
board to target. Setting a board in practice amounts to generating a
`.cargo/board.toml` specifying the appropriate target JSON and select
build-internal environment variables, which take effect by virtue of an
inclusion of this file in `.cargo/config.toml`.

One finds two kinds of board-specific, build-internal configuration files under
`hark/hark/board/`: one is the board-specific target JSON (`foo.json`), which
tells `rustc` how to compile for the target; the other is our own internal board
TOML configuration file (`foo.toml`) that gets ingested by hark's custom build
script (e.g., to enable certain cfg keys or define particular symbols), and is
discovered through the `HARK_BOARD` environment variable.

Another aim of this project was to make hark "app" development simply a Cargo
`bin` with a `hark_app_main()` entrypoint that simply depends on the `hark`
crate. This was mostly achieved. Cargo limitations however force the following
concessions:

- an app must have a custom build script that invokes
  `hark_build::declare_app()` (which ensures that important linker arguments get
  passed);
- a project must replicate the target JSON files themselves.

## Getting started

These are 'in-tree' instructions, since the out-of-tree flow doesn't seem
fleshed out enough yet.

- Supported boards can be listed with `scripts/set-board.nu` (no arguments).

- A supported board must be set first with `scripts/set-board.nu $board`.

- A `bin` is defined with a `hark_app_main()` entrypoint, a dependency on the
  hark crate, and a custom build script that calls `hark_build::declare_app()`.

- `cargo build` produces an ELF version of the app, whose read-only segments are
  expected to be loaded out of flash and whose writable segments are expected to
  be copied into RAM during execution.

- `scripts/qemu.nu` will run that app under QEMU (as a raw binary), in a
  configuration intended to match the current board as much as possible. (Well,
  actually it just runs the `example` app today, but that can be easily
  tweaked.)
