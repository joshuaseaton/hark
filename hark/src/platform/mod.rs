// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

pub mod console;

#[cfg_attr(platform = "qemu-virt-riscv", path = "qemu_virt_riscv.rs")]
mod backend;

/// Shuts down the system in an orderly manner.
pub fn shutdown() -> ! {
    backend::shutdown()
}

/// Shuts down the system in the event of an unreliable kernel state.
pub fn halt() -> ! {
    backend::halt()
}

/// Reboots the system.
pub fn reboot() -> ! {
    backend::reboot()
}
