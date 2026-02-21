// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use crate::dev::sifive_test;

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

#[inline]
pub fn shutdown() -> ! {
    sifive_test::shutdown()
}

#[inline]
pub fn halt() -> ! {
    sifive_test::panic()
}

#[inline]
pub fn reboot() -> ! {
    sifive_test::reset();
}
