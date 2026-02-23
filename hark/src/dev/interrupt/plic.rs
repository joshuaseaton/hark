// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

//
// The RISC-V Platform-Level Interrupt Controller
//

use derive_more::{Deref, From};
use regio::{Arrayed, FixedAddr, Mmio, Offset, Readable, Register, Writable, array};

use crate::dev::interrupt::ControllerBase;

const APERTURE_SIZE: usize = 0x400_0000;

type Io = Mmio<u32>;

trait BitArray:
    Register<Base = u32, Addr = Offset> + Readable + Writable + Arrayed + FixedAddr
{
    fn get(mmio: &Io, n: usize) -> bool {
        let block_num = n / (u32::BITS as usize);
        let block = *Self::read_nth_from(mmio, block_num);
        let index = n % 32;
        ((block >> index) & 1) != 0
    }

    fn set(mmio: &Io, n: usize, value: bool) {
        let block_num = n / (u32::BITS as usize);
        let current = Self::read_nth_from(mmio, block_num);
        let index = n % 32;
        let shifted = 1u32 << index;

        let updated = if value {
            Self::from(*current | shifted)
        } else {
            Self::from(*current & !shifted)
        };
        updated.write_nth_to(mmio, block_num);
    }
}

// PLIC target context, a numeric scheme corresponding to (M/S-mode, hart ID)
// pairs.
#[derive(Clone, Copy, Debug, Deref)]
pub struct Context(usize);

// The interrupt priority for each interrupt source.
#[array(0x0)]
#[derive(Debug, Deref, From)]
struct InterruptSourcePriority(u32);

// The interrupt pending status of each interrupt source.
#[array(0x1000)]
#[derive(Debug, Deref, From)]
struct InterruptPendingBits(u32);

// The enablement of interrupt source of each context.
#[array(0x2000)]
#[derive(Debug, Deref, From)]
struct InterruptEnableBits(u32);

impl BitArray for InterruptEnableBits {}

impl InterruptEnableBits {
    const BITS_PER_CONTEXT: usize = 1024;

    fn get_for_context(mmio: &Io, context: Context, n: usize) -> bool {
        Self::get(mmio, *context * Self::BITS_PER_CONTEXT + n)
    }

    fn set_for_context(mmio: &Io, context: Context, n: usize, value: bool) {
        Self::set(mmio, *context * Self::BITS_PER_CONTEXT + n, value);
    }
}

// The interrupt priority threshold of each context.
#[array(0x20_0000, stride = 0x1000)]
#[derive(Debug, Deref, From)]
struct PriorityThreshold(u32);

// The register to acquire interrupt source ID of each context (on read), as
// well as the register to send interrupt completion messages to the associated
// gateway (on write).
#[array(0x20_0004, stride = 0x1000)]
#[derive(Debug, Deref, From)]
struct InterruptClaimProcess(u32);

pub struct Plic {}

impl ControllerBase for Plic {
    type Io = Io;

    type TargetId = Context;

    #[inline]
    fn cpu_number_to_target_id(cpu_num: u32) -> Context {
        let cpu_num = cpu_num as usize;
        if cfg!(riscv_m_mode) {
            Context(2 * cpu_num)
        } else {
            Context(2 * cpu_num + 1)
        }
    }

    #[inline]
    fn io(base: usize) -> Io {
        Mmio::new(base, APERTURE_SIZE)
    }

    #[inline]
    fn init_global(io: &Io, max_irq: u32) {
        // Set the priority to every interrupt to 1.
        for i in 0..max_irq {
            InterruptSourcePriority::from(1).write_nth_to(io, i as usize);
        }
    }

    #[inline]
    fn init_target(io: &Io, context: Context, max_irq: u32) {
        // Disable all interrupts for the context by zeroing enable words
        // directly, and set the priority threshold to zero (so that nothing
        // is fixed as masked).
        let words = max_irq.div_ceil(u32::BITS) as usize;
        let base = *context * (InterruptEnableBits::BITS_PER_CONTEXT / u32::BITS as usize);
        for i in 0..words {
            InterruptEnableBits::from(0).write_nth_to(io, base + i);
        }
        PriorityThreshold::from(0).write_nth_to(io, *context);
    }

    fn enable_irq(io: &Io, context: Context, irq: u32) {
        InterruptEnableBits::set_for_context(io, context, irq as usize, true);
    }

    fn claim_pending_irq(io: &Io, context: Context) -> Option<u32> {
        let irq = *InterruptClaimProcess::read_nth_from(io, *context);
        (irq > 0).then_some(irq)
    }

    fn complete_irq(io: &Io, context: Context, irq: u32) {
        InterruptClaimProcess::from(irq).write_nth_to(io, *context);
    }
}
