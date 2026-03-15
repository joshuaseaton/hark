// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use super::Manager;

/// Used only during bring-up as a convenient placeholder.
pub struct PowerManager {}

impl Manager for PowerManager {
    fn shutdown() -> ! {
        loop {}
    }
    fn halt() -> ! {
        loop {}
    }
    fn reboot() -> ! {
        loop {}
    }
}
