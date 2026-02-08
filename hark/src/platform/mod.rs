// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

cfg_if::cfg_if! {
    if #[cfg(platform = "qemu-virt-riscv")] {
        mod qemu_virt_riscv;
        use qemu_virt_riscv::Impl;
    }
}

trait Platform {
    fn shutdown() -> !;
    fn halt() -> !;
    fn reboot() -> !;
}

/// Shuts down the system in an orderly manner.
pub fn shutdown() -> ! {
    Impl::shutdown()
}

/// Shuts down the system in the event of an unreliable kernel state.
pub fn halt() -> ! {
    Impl::halt()
}

/// Reboots the system.
pub fn reboot() -> ! {
    Impl::reboot()
}
