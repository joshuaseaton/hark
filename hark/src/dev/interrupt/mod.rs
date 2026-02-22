// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

mod plic;

use regio::IoBackend;

use crate::arch;

pub type Plic = Controller<plic::Plic>;

// Abstract interrupt controller traits.
pub trait ControllerBase {
    type Io: IoBackend;

    // The identity of an interrupt target, a device-specific concept that
    // corresponds to a CPU (or CPU context) through an arch-/platform-specific
    // mapping.
    type TargetId;

    fn cpu_number_to_target_id(cpu_num: u32) -> Self::TargetId;

    fn io(base: usize) -> Self::Io;

    // The global, non-target-specific initialization routine.
    fn init_global(io: &Self::Io, max_irq: u32);

    // The locaal, target-specific initialization routine.
    fn init_target(io: &Self::Io, target: Self::TargetId, max_irq: u32);

    fn enable_irq(io: &Self::Io, target: Self::TargetId, irq: u32);

    fn claim_irq(io: &Self::Io, target: Self::TargetId) -> Option<u32>;

    fn complete_irq(io: &Self::Io, target: Self::TargetId, irq: u32);
}

// A generic interrupt controller.
pub struct Controller<Base: ControllerBase> {
    io: Base::Io,
}

impl<Base: ControllerBase> Controller<Base> {
    // Creates and initializes an interrupt controller, including initialization
    // for the local CPU.
    pub fn new(base: usize, max_irq: u32) -> Self {
        let io = Base::io(base);
        Base::init_global(&io, max_irq);
        Base::init_target(&io, Self::local_id(), max_irq);
        Self { io }
    }

    // The local target ID.
    fn local_id() -> Base::TargetId {
        Base::cpu_number_to_target_id(arch::current_cpu_number())
    }

    pub fn enable_irq(&self, irq: u32) {
        Base::enable_irq(&self.io, Self::local_id(), irq);
    }

    pub fn claim_irq(&self) -> Option<u32> {
        Base::claim_irq(&self.io, Self::local_id())
    }

    pub fn complete_irq(&self, irq: u32) {
        Base::complete_irq(&self.io, Self::local_id(), irq);
    }
}
