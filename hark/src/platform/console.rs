// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::fmt;
use core::mem::MaybeUninit;

use crate::platform::{Console, backend};

static mut CONSOLE: MaybeUninit<backend::Console> = MaybeUninit::uninit();

pub(super) fn set_console(console: backend::Console) {
    unsafe {
        (*&raw mut CONSOLE).write(console);
    }
}

pub(super) fn get_console() -> &'static impl Console {
    unsafe { (*&raw const CONSOLE).assume_init_ref() }
}

pub(crate) fn describe(w: &mut impl fmt::Write) {
    get_console().describe(w);
}

pub(crate) fn init() {
    set_console(backend::console());
}

/// Writes to the platform-defined console.
pub fn write(bytes: &[u8]) {
    get_console().write(bytes);
}
