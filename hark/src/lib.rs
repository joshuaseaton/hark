// Copyright (c) 2025 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

#![no_std]

pub mod arch;
mod dev;
pub mod kernel;
pub mod platform;

use kernel::debug::build_id;

use core::fmt::{self, Write as _};
use core::panic::PanicInfo;
use core::write;

const HARK_WELCOME: &str = r"
▄▄  ▄▄ 
██  ██             ▄▄
██▄▄██  ▀▀█▄ ▄███▄ ██ ▄█▀
██▀▀██ ▄█▀██ ██ ▀▀ ████ 
██  ██ ▀█▄██ ██    ██ ▀█▄
";

const HARK_GOODBYE: &str = r"
 ▄▄▄▄                          ▄▄
██▀▀██                       ▄█▀▀█▄
██      ▀▀█▄ ███▄███▄ ▄█▀█▄  ██. ██ ██ ██ ▄█▀█▄ ▄███▄
██ ▀██ ▄█▀██ ██ ██ ██ ██▄█▀  ██  ██ ██▄██ ██▄█▀ ██ ▀▀
▀████▀ ▀█▄██ ██ ██ ██ ▀█▄▄▄   ▀██▀   ▀█▀  ▀█▄▄▄ ██
";

/// A conventional "stdout", backed by the platform console.
pub struct Stdout {}

impl fmt::Write for Stdout {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        platform::console_write(s.as_bytes());
        Ok(())
    }
}

/// Prints to the platform console.
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        write!($crate::Stdout {}, $($arg)*).unwrap();
    };
}

/// Prints to the platform console, with a newline.
#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        print!($($arg)*);
        print!("\n");
    };
}

// Jumped to from _start after initialization.
#[unsafe(no_mangle)]
extern "C" fn hark_main() {
    println!("{HARK_WELCOME}");
    println!(
        "Version: {} ({})",
        env!("HARK_VERSION"),
        env!("HARK_REVISION")
    );
    print!("Build ID: {}", build_id());

    // Nothing more yet to do.
    panic!("this panic was intentional");
}

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    println!("{HARK_GOODBYE}");
    println!("{info}");

    // Nothing more yet to do.
    platform::halt()
}
