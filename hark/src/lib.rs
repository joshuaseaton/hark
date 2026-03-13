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
pub mod shell;
pub mod sync;
pub mod thread;

mod panic;
pub(crate) use panic::*;

// Allows #[shell::command] works within hark, since its expansion includes item
// paths under `::hark::shell`.
#[doc(hidden)]
extern crate self as hark;

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

// Initialization routines can be interdependent. To make that dependency more
// explicit, we introduce "witness" types that represent a dependent
// initialization: steps that perform the dependent initialization return an
// instance of the witness; steps that depend on such an initialization take the
// witness type as a parameter.

// For init routines that assume the ability to print.
pub(crate) struct ConsoleWitness {}

// For init routines that assume threading.
pub(crate) struct ThreadWitness {}

// Jumped to from _start after initialization.
#[unsafe(no_mangle)]
extern "C" fn hark_main() {
    let console = platform::early_init(); // Initializes the console
    print_welcome(&console);
    print_version(&console);

    // Parses the build ID. Do it early for symbolizable backtraces.
    debug::init(&console);
    print_build_id(&console);
    print_boot_memory(&console);
    print_console_info(&console);

    arch::init(&console);
    platform::init(&console);
    heap::init();
    let thread = thread::init();

    arch::late_init(&thread);

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
fn print_welcome(_: &ConsoleWitness) {
    println!("{HARK_WELCOME}");
}

#[inline(never)]
fn print_version(_: &ConsoleWitness) {
    println!(
        "Version: {} ({})",
        env!("HARK_VERSION"),
        env!("HARK_REVISION")
    );
}

#[inline(never)]
fn print_build_id(_: &ConsoleWitness) {
    println!("Build ID: {}", build_id());
}

#[inline(never)]
fn print_console_info(_: &ConsoleWitness) {
    print!("Console: ");
    platform::console::describe(&mut Stdout {});
    print!("\n");
}

#[inline(never)]
fn print_boot_memory(_: &ConsoleWitness) {
    let flash_start = (&raw const __boot_flash_start).addr();
    let flash_end = (&raw const __boot_flash_end).addr();
    println!("Boot flash: [{flash_start:#x}, {flash_end:#x})");

    let ram_start = (&raw const __boot_ram_start).addr();
    let ram_end = (&raw const __boot_ram_end).addr();
    println!("Boot RAM: [{ram_start:#x}, {ram_end:#x})");
}
