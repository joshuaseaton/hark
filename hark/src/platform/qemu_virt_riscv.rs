// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use crate::dev::sifive_test;
use crate::platform::Platform;

#[cfg(all(riscv_sbi_console, riscv_m_mode))]
compile_error!("Can't use an SBI console in M mode");

cfg_if::cfg_if! {
    if #[cfg(riscv_sbi_console)] {
        use crate::arch::riscv::SbiConsole;
    } else {
        use crate::dev::uart;

        const UART_ADDR: usize = 0x1000_0000;
    }
}

pub(super) struct Impl {}

impl Platform for Impl {
    cfg_if::cfg_if! {
        if #[cfg(riscv_sbi_console)] {
            type Console = SbiConsole;
        } else {
            type Console = uart::Ns8250;
        }
    }

    #[inline]
    fn console() -> Self::Console {
        cfg_if::cfg_if! {
            if #[cfg(riscv_sbi_console)] {
                SbiConsole{}
            } else {
                uart::Ns8250::new(UART_ADDR)
            }
        }
    }

    #[inline]
    fn shutdown() -> ! {
        sifive_test::shutdown()
    }

    #[inline]
    fn halt() -> ! {
        sifive_test::panic()
    }

    #[inline]
    fn reboot() -> ! {
        sifive_test::reset();
    }
}
