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

pub(crate) fn init_post_console() {
    interrupt::init();
    memory::init(&backend::MEMORY_MAP);
}
