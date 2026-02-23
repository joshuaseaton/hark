// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use crate::dev::interrupt::Plic;
use crate::dev::power::SiFiveTest;
use crate::platform::memory;

#[cfg(not(any(target_arch = "riscv32", target_arch = "riscv64")))]
compile_error!("qemu-virt-riscv is only defined for RISC-V");

#[cfg(all(riscv_sbi_console, riscv_m_mode))]
compile_error!("Can't use an SBI console in M mode");

cfg_if::cfg_if! {
    if #[cfg(riscv_sbi_console)] {
        use crate::arch::riscv::SbiDebugConsole;

        pub type Console = SbiDebugConsole;

        #[inline]
        pub fn console() -> Console { SbiDebugConsole{}}
    } else {
        use crate::dev::uart;

        const UART_ADDR: usize = 0x1000_0000;

        pub type Console = uart::Ns8250;

        #[inline]
        pub fn console() -> Console { uart::Ns8250::new(UART_ADDR) }
    }
}

const PLIC_ADDR: usize = 0x0c00_0000;
pub const MAX_IRQ: u32 = 96;

pub const MEMORY_MAP: [memory::Range; 1] = [
    // TODO: size based on compile-time configuration?
    memory::Range {
        start: 0x8000_0000,
        size: 0x800_0000,
    },
];

pub type PowerManager = SiFiveTest;

pub type InterruptController = Plic;

pub fn interrupt_controller() -> Plic {
    Plic::new(PLIC_ADDR, MAX_IRQ)
}
