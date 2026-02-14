// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::fmt;
use core::mem::MaybeUninit;

use crate::dev::uart;
use crate::platform::{Impl, Platform};

type PlatformConsole = <Impl as Platform>::Console;
static mut CONSOLE: MaybeUninit<PlatformConsole> = MaybeUninit::uninit();

pub(super) fn set_console(console: PlatformConsole) {
    unsafe {
        (*&raw mut CONSOLE).write(console);
    }
}

pub(super) fn get_console() -> &'static PlatformConsole {
    unsafe { (*&raw const CONSOLE).assume_init_ref() }
}

pub(crate) trait Console {
    fn describe(&self, w: &mut impl fmt::Write);
    fn write(&self, bytes: &[u8]);
}

impl<Base: uart::DriverBase> Console for uart::Driver<Base> {
    #[inline]
    fn describe(&self, w: &mut impl fmt::Write) {
        uart::Driver::describe(self, w);
    }

    #[inline]
    fn write(&self, bytes: &[u8]) {
        uart::Driver::write(self, bytes);
    }
}
