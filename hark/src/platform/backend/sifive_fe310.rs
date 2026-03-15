// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use crate::dev::interrupt::Plic;
use crate::dev::power::NullPowerManager;
use crate::dev::uart;
use crate::heap;

#[cfg(not(target_arch = "riscv32"))]
compile_error!("sifive-fe310 is only defined for RV32");

// A required part of the pub(crate) API for RISC-V platforms.
pub const RISCV_MTIMER_TIME_ADDRESS: usize = 0x0200_bff8;
pub const RISCV_MTIMER_TIMECMP_ADDRESS: usize = 0x0200_4000;

const UART_ADDR: usize = 0x1001_3000;
const PLIC_ADDR: usize = 0x0c00_0000;
pub const MAX_IRQ: u32 = 52;

pub const TIMER_FREQUENCY: u64 = 32_768; // 32.768 kHz

pub const RAM: heap::Range = heap::Range {
    start: 0x8000_0000,
    size: 0x0000_4000, // 16KiB
};

pub type Console = uart::SiFive;

pub type InterruptController = Plic;

// TODO: Implement real power management.
pub type PowerManager = NullPowerManager;

#[inline]
pub fn console() -> Console {
    uart::SiFive::new(UART_ADDR)
}

#[inline]
pub fn interrupt_controller() -> Plic {
    Plic::new(PLIC_ADDR, MAX_IRQ)
}
