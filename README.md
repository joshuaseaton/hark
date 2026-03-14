# hark

*hark* ("**H**ere, **a**nother **R**ust **k**ernel") is an embedded Rust
microkernel.

Intended to be a flexible microkernel for embedded, single code SoCs. Will
feature extensible built-in support for (hopefully) a wide array of RISC-V
and Cortex-M SoCs, and will be straightforward to port to new ones.

**Reality check**: currently we only support QEMU's RISC-V virt machine.

## Features (so far):
* Cargo-native app model: just have you bin depend on the hark crate! (well, and
set a few environment variables);
* Backtraces with offline symbolization, courtesy of the LLVM symbolizer markup;
* A shell over serial, and a simple framework to contribute custom shell
commands (see `#[hark::shell::command]`);
* A testing framework, where additional tests (beyond hark's own) can be
contributed from anywhere in the app's crate (see `#[hark::test]`);
* Cooperative and pre-emptive multitasking;

TODO: Make hark do more, and say more about it!
