// Copyright (c) 2025 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

cfg_select! {
    any(target_arch = "riscv32", target_arch = "riscv64") => {
        mod riscv;
        use riscv as backend;
    }
    _ => {}
}

pub(crate) use backend::thread;

use crate::{ConsoleWitness, ThreadWitness};

pub(crate) fn init(console: &ConsoleWitness) {
    backend::init(console);
}

pub(crate) fn late_init(thread: &ThreadWitness) {
    backend::late_init(thread);
}

// Used in sync::InterruptGuard.
pub(crate) use backend::{InterruptSaveState, restore_interrupt_state, save_interrupt_state};

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
