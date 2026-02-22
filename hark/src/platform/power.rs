// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use crate::dev::power::Manager as _;
use crate::platform::backend;

/// Shuts down the system in an orderly manner.
pub fn shutdown() -> ! {
    backend::PowerManager::shutdown()
}

/// Shuts down the system in the event of an unreliable kernel state.
pub fn halt() -> ! {
    backend::PowerManager::halt()
}

/// Reboots the system.
pub fn reboot() -> ! {
    backend::PowerManager::reboot()
}
