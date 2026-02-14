// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use crate::dev::sifive_test;
use crate::platform::Platform;

cfg_if::cfg_if! {
    if #[cfg(riscv_m_mode)] {
        use crate::dev::uart;

        const UART_ADDR: usize = 0x1000_0000;
    } else {
        use crate::arch::riscv::SbiConsole;
    }
}

pub(super) struct Impl {}

impl Platform for Impl {
    cfg_if::cfg_if! {
        if #[cfg(riscv_m_mode)] {
            type Console = uart::Ns8250;
        } else {
            type Console = SbiConsole;
        }
    }

    #[inline]
    fn console() -> Self::Console {
        cfg_if::cfg_if! {
            if #[cfg(riscv_m_mode)] {
                uart::Ns8250::new(UART_ADDR)
            } else {
                SbiConsole{}
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
