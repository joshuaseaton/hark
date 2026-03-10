// Copyright (c) 2025 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

#![no_std]

pub mod arch;
pub mod debug;
pub mod heap;
pub mod platform;
mod shell;
pub mod thread;

mod panic;
pub(crate) use panic::*;

// There is naturally going to be a lot of dead device code in any given
// configuration, so the lint here would be too noisy. We can at least permit
// the lint in clippy's analysis and downgrade the warning to a hint with
// the rust-analyzer.diagnostics.warningsAsHint option. This keeps dead code as
// greyed out in the editor, but not with squiggles.
#[cfg_attr(not(clippy), allow(dead_code))]
pub(crate) mod dev;

use core::fmt;

use debug::build_id;

unsafe extern "C" {
    static __boot_flash_start: u8;
    static __boot_flash_end: u8;
    static __boot_ram_start: u8;
    static __boot_ram_end: u8;
}

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
        platform::console::write(s.as_bytes());
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

// A Hark app must define this.
unsafe extern "Rust" {
    fn hark_app_main();
}

// Jumped to from _start after initialization.
#[unsafe(no_mangle)]
extern "C" fn hark_main() {
    platform::console::init();
    print_welcome();
    print_version();

    // Parses the build ID. Do it early for symbolizable backtraces.
    debug::early_init();
    print_build_id();

    print_boot_memory();
    print_console_info();

    arch::init();
    platform::init_post_console();
    heap::init();
    thread::init();

    shell::run_in_background();

    unsafe {
        hark_app_main();
    }
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
    platform::console::describe(&mut Stdout {});
    print!("\n");
}

#[inline(never)]
fn print_boot_memory() {
    let flash_start = (&raw const __boot_flash_start).addr();
    let flash_end = (&raw const __boot_flash_end).addr();
    println!("Boot flash: [{flash_start:#x}, {flash_end:#x})");

    let ram_start = (&raw const __boot_ram_start).addr();
    let ram_end = (&raw const __boot_ram_end).addr();
    println!("Boot RAM: [{ram_start:#x}, {ram_end:#x})");
}
