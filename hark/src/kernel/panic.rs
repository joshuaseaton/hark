// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::panic::PanicInfo;

use crate::kernel::debug::print_backtrace;
use crate::platform;
use crate::println;

const HARK_GOODBYE: &str = r"
 ▄▄▄▄                          ▄▄
██▀▀██                       ▄█▀▀█▄
██      ▀▀█▄ ███▄███▄ ▄█▀█▄  ██. ██ ██ ██ ▄█▀█▄ ▄███▄
██ ▀██ ▄█▀██ ██ ██ ██ ██▄█▀  ██  ██ ██▄██ ██▄█▀ ██ ▀▀
▀████▀ ▀█▄██ ██ ██ ██ ▀█▄▄▄   ▀██▀   ▀█▀  ▀█▄▄▄ ██
";

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    panic_common(libarch::frame_pointer(), None, || print_panic_info(info));
}

// Common panic routine for use within panic!() and in exception handling.
pub(crate) fn panic_common<PrintContext: FnOnce()>(
    fp: usize,
    pc: Option<usize>,
    context: PrintContext,
) -> ! {
    print_goodbye();
    context();
    print_backtrace(fp, pc);

    // Nothing more yet to do.
    platform::power::halt()
}

#[inline(never)]
fn print_goodbye() {
    println!("{HARK_GOODBYE}");
}

#[inline(never)]
fn print_panic_info(info: &PanicInfo) {
    println!("{info}");
}
