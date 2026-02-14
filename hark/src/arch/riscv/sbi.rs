// Copyright (c) 2025 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::fmt;

use libarch::riscv::{SbiError, sbi_debug_console_write};

use crate::platform::Console;

#[allow(unused)]
pub(crate) struct SbiConsole {}

impl Console for SbiConsole {
    fn describe(&self, w: &mut impl fmt::Write) {
        let _ = write!(w, "SBI debug console");
    }

    fn write(&self, bytes: &[u8]) {
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
