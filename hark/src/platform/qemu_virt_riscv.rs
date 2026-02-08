// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use super::Platform;
use crate::dev::sifive_test;

use libarch::riscv::{SbiError, sbi_debug_console_write};

pub(super) struct Impl {}

impl Platform for Impl {
    fn shutdown() -> ! {
        sifive_test::shutdown()
    }

    fn halt() -> ! {
        sifive_test::panic()
    }

    fn reboot() -> ! {
        sifive_test::reset();
    }

    fn console_write(bytes: &[u8]) {
        let mut remaining = bytes.len();
        while remaining > 0 {
            match sbi_debug_console_write(bytes) {
                Ok(written) => remaining -= written,
                Err(SbiError::DENIED) => break,
                _ => {}
            }
        }
    }
}
