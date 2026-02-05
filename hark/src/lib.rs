// Copyright (c) 2025 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

#![no_std]

mod arch;

use core::panic::PanicInfo;

// Jumped to from _start after initialization.
#[unsafe(no_mangle)]
extern "C" fn hark_main() {
    // TODO: Do more.
    loop {}
}

#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
