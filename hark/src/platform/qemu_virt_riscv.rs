// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use crate::dev::interrupt::Plic;
use crate::dev::power::SiFiveTest;
use crate::dev::uart;
use crate::platform::memory;

#[cfg(not(any(target_arch = "riscv32", target_arch = "riscv64")))]
compile_error!("qemu-virt-riscv is only defined for RISC-V");

// A required part of the pub(crate) API for RISC-V platforms.
pub const RISCV_MTIMER_TIME_ADDRESS: usize = 0x0200_bff8;
pub const RISCV_MTIMER_TIMECMP_ADDRESS: usize = 0x0200_4000;

const UART_ADDR: usize = 0x1000_0000;
const PLIC_ADDR: usize = 0x0c00_0000;
pub const MAX_IRQ: u32 = 96;

pub const MEMORY_MAP: [memory::Range; 1] = [
    // TODO: size based on compile-time configuration?
    memory::Range {
        start: 0x8000_0000,
        size: 0x800_0000,
    },
];

pub type Console = uart::Ns8250;

pub type InterruptController = Plic;

pub type PowerManager = SiFiveTest;

#[inline]
pub fn console() -> Console {
    uart::Ns8250::new(UART_ADDR)
}

#[inline]
pub fn interrupt_controller() -> Plic {
    Plic::new(PLIC_ADDR, MAX_IRQ)
}
