// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

pub mod console;
pub mod interrupt;
mod memory;
pub mod power;

#[cfg_attr(platform = "qemu-virt-riscv", path = "qemu_virt_riscv.rs")]
mod backend;

cfg_if::cfg_if! {
    if #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))] {
        // While the CLINT/ACLINT MTIMER is a feature of the RISC-V
        // architecture, its MMIO addresses areplatform-defined. So we violate
        // the arch-platform layering a teensy amount to export these values
        // back down.
        pub(crate) use backend::RISCV_MTIMER_TIME_ADDRESS;
        pub(crate) use backend::RISCV_MTIMER_TIMECMP_ADDRESS;
    }
}

pub(crate) fn init_post_console() {
    interrupt::init();
    memory::init(&backend::MEMORY_MAP);
}
