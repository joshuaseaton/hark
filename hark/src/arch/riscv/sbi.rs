// Copyright (c) 2025 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::fmt;

use libarch::riscv::sbi;

use crate::dev::Console;

pub(crate) struct SbiDebugConsole {}

impl Console for SbiDebugConsole {
    fn describe(&self, w: &mut impl fmt::Write) {
        let _ = write!(w, "SBI debug console");
    }

    fn write(&self, mut bytes: &[u8]) {
        while !bytes.is_empty() {
            match sbi::debug_console_write(bytes) {
                Ok(written) => bytes = &bytes[written..],
                Err(sbi::Error::DENIED) => break,
                _ => {}
            }
        }
    }

    fn read_byte(&self) -> Option<u8> {
        let byte = 0u8;
        let written = sbi::debug_console_read(&mut [byte]).ok()?;
        (written == 1).then_some(byte)
    }

    fn read(&self, buffer: &mut [u8]) -> usize {
        sbi::debug_console_read(buffer).ok().unwrap_or(0)
    }
}
