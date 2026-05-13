// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

pub mod console;
pub(crate) mod interrupt;
pub mod power;
mod time;

pub use time::Time;

#[cfg_attr(platform = "qemu-virt-riscv", path = "backend/qemu_virt_riscv.rs")]
#[cfg_attr(platform = "sifive-fe310", path = "backend/sifive_fe310.rs")]
mod backend;

cfg_select! {
    any(target_arch = "riscv32", target_arch = "riscv64") => {
        // While the CLINT/ACLINT MTIMER is a feature of the RISC-V
        // architecture, its MMIO addresses are platform-defined. So we violate
        // the arch-platform layering by a teensy amount to export these values
        // back down.
        pub(crate) use backend::RISCV_MTIMER_TIME_ADDRESS;
        pub(crate) use backend::RISCV_MTIMER_TIMECMP_ADDRESS;
    }
    _ => {}
}

// TODO: Generalize this to a list when needed.
pub(crate) use backend::RAM;

use crate::ConsoleWitness;

pub(crate) fn early_init() -> ConsoleWitness {
    console::init();
    ConsoleWitness {}
}

pub(crate) fn init(_: &ConsoleWitness) {
    interrupt::init();
}
