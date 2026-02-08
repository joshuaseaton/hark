// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use super::Platform;
use crate::dev::sifive_test;

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
}
