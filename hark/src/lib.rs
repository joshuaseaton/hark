// Copyright (c) 2025 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

#![no_std]

pub mod arch;
pub mod kernel;
pub mod platform;

// There is naturally going to be a lot of dead device code in any given
// configuration, so the lint here would be too noisy. We can at least permit
// the lint in clippy's analysis and downgrade the warning to a hint with
// the rust-analyzer.diagnostics.warningsAsHint option. This keeps dead code as
// greyed out in the editor, but not with squiggles.
#[cfg_attr(not(clippy), allow(dead_code))]
pub(crate) mod dev;

use core::fmt;

use kernel::debug::build_id;

const HARK_WELCOME: &str = r"
▄▄  ▄▄ 
██  ██             ▄▄
██▄▄██  ▀▀█▄ ▄███▄ ██ ▄█▀
██▀▀██ ▄█▀██ ██ ▀▀ ████ 
██  ██ ▀█▄██ ██    ██ ▀█▄
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
    ($($arg:tt)*) => {{
        use core::fmt::Write as _;
        write!($crate::Stdout {}, $($arg)*).unwrap()
    }};
}

/// Prints to the platform console, with a newline.
#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {{
        $crate::print!($($arg)*);
        $crate::print!("\n")
    }};
}

// Jumped to from _start after initialization.
#[unsafe(no_mangle)]
extern "C" fn hark_main() {
    platform::console_init();
    print_welcome();
    print_version();

    // Parses the build ID. Do it early for symbolizable backtraces.
    kernel::debug::early_init();
    print_build_id();

    print_console_info();

    arch::print_machine_context();

    // Nothing more yet to do.
    panic!("this panic was intentional");
}

//
// Printing is stack-hungry, so we put these print routines in inline(never)
// wrappers to avoid overflows.
//

#[inline(never)]
fn print_welcome() {
    println!("{HARK_WELCOME}");
}

#[inline(never)]
fn print_version() {
    println!(
        "Version: {} ({})",
        env!("HARK_VERSION"),
        env!("HARK_REVISION")
    );
}

#[inline(never)]
fn print_build_id() {
    println!("Build ID: {}", build_id());
}

#[inline(never)]
fn print_console_info() {
    print!("Console: ");
    platform::console_describe(&mut Stdout {});
    print!("\n");
}
