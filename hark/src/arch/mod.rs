// Copyright (c) 2025 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

cfg_if::cfg_if! {
    if #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))] {
        mod riscv;
        use riscv as backend;
    }
}

pub(crate) use backend::thread;

use crate::ConsoleWitness;

pub(crate) fn init(console: &ConsoleWitness) {
    backend::init(console);
}

/// Returns the current CPU number.
///
/// For RISC-V this is the hart ID.
pub fn current_cpu_number() -> u32 {
    backend::current_cpu_number()
}

/// Returns the current time in CPU ticks.
pub fn get_ticks() -> u64 {
    backend::get_ticks()
}

/// Enables interrupts.
pub fn enable_interrupts() {
    backend::enable_interrupts();
}

/// Disables interrupts.
pub fn disable_interrupts() {
    backend::disable_interrupts();
}
