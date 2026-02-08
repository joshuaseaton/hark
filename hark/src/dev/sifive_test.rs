// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

// The "SiFive Test" device (A.K.A. "Test Finisher") is a simple, virtual MMIO
// device contributed to QEMU, and in use in the RISC-V QEMU virt machine. It
// featurs a single register that can be written to exit or restart the guest.

// Documented only in qemu.git source:
// https://github.com/qemu/qemu/blob/cd5a79dc98e3087e7658e643bdbbb0baec77ac8a/include/hw/misc/sifive_test.h

use core::ptr;

// The one and only register.
const ADDR: usize = 0x10_0000;

// Special values to write to ADDR.
const FAIL: u32 = 0x3333;
const PASS: u32 = 0x5555;
const RESET: u32 = 0x7777;

// Terminates QEMU virt machine guest with an indication to QEMU that it shut
// down in an orderly manner.
pub(crate) fn shutdown() -> ! {
    unsafe {
        *(ptr::without_provenance_mut(ADDR)) = PASS;
    }
    loop {}
}

// Terminates a QEMU virt machine guest with an indication to QEMU that it shut
// down in an emergency.
pub(crate) fn panic() -> ! {
    unsafe {
        *(ptr::without_provenance_mut(ADDR)) = FAIL;
    }
    loop {}
}

// Reboots a QEMU virt machine guest.
pub(crate) fn reset() -> ! {
    unsafe {
        *(ptr::without_provenance_mut(ADDR)) = RESET;
    }
    loop {}
}
