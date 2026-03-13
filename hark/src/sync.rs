// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::marker::PhantomData;

use crate::arch;

/// An RAII guard that ensures that interrupts are disabled.
///
/// Interrupts are disabled on construction and the original interrupt state is
/// restored on drop. The restoring of the original state - and not blind
/// re-enabling - ensures that this may be used in interrupt and non-interrupt
/// contexts alike, and tolerates nesting.
#[must_use]
pub struct InterruptGuard {
    state: arch::InterruptSaveState,
    _phantom: PhantomData<*const ()>, // Ensures !Send and !Sync
}

impl InterruptGuard {
    pub fn new() -> Self {
        Self {
            state: arch::save_interrupt_state(),
            _phantom: PhantomData,
        }
    }
}

impl Drop for InterruptGuard {
    fn drop(&mut self) {
        arch::restore_interrupt_state(self.state);
    }
}
