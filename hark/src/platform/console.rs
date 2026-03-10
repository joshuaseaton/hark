// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::fmt;
use core::mem::MaybeUninit;

use crate::dev::Console;
use crate::platform::backend;

static mut CONSOLE: MaybeUninit<backend::Console> = MaybeUninit::uninit();

fn set_console(console: backend::Console) {
    unsafe {
        (*&raw mut CONSOLE).write(console);
    }
}

fn get_console() -> &'static impl Console {
    unsafe { (*&raw const CONSOLE).assume_init_ref() }
}

pub(crate) fn describe(w: &mut impl fmt::Write) {
    get_console().describe(w);
}

pub(super) fn init() {
    set_console(backend::console());
}

/// Writes to the platform-defined console.
pub fn write(bytes: &[u8]) {
    get_console().write(bytes);
}

// Writs a byte to the platform-defined console.
pub fn write_byte(byte: u8) {
    write(&[byte; 1]);
}

/// Reads a byte from the platform-defined console (non-blocking).
pub fn read_byte() -> Option<u8> {
    get_console().read_byte()
}

/// Reads from the platform-defined console into a provided buffer
/// (non-blocking), returning the number of bytes read in.
pub fn read(buffer: &mut [u8]) -> usize {
    get_console().read(buffer)
}
