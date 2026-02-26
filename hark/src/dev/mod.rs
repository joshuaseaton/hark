// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

pub mod interrupt;
pub mod power;
pub mod uart;

use core::fmt;

pub(crate) trait Console {
    fn describe(&self, w: &mut impl fmt::Write);
    fn write(&self, bytes: &[u8]);

    // Does not block. Returns None if there is no data to read.
    fn read_byte(&self) -> Option<u8>;

    // Reads in as much as is present and can be written to the provided
    // buffer. This method also does not block.
    fn read(&self, bytes: &mut [u8]) -> usize {
        for (i, byte) in bytes.iter_mut().enumerate() {
            let Some(ch) = self.read_byte() else {
                return i;
            };
            *byte = ch;
        }
        bytes.len()
    }
}
