// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

#![no_std]
#![no_main]

use hark::shell;
use hark::testing;
use hark::{println, test};

#[unsafe(no_mangle)]
extern "Rust" fn hark_app_main() {
    println!("Hello from a Hark app!");
    loop {}
}

//
// Example shell command.
//

/// This is a description of a custom shell command!
#[shell::command(help = "This is a custom command")]
fn custom(_: shell::Args) -> bool {
    println!("The custom command was called!");
    true
}
//
// Example tests.
//

#[test]
fn sanity() -> Result<(), testing::Failure> {
    testing::check_eq!(1 + 1, 2);
    Ok(())
}

#[allow(clippy::const_is_empty)]
#[test]
fn fail_demo() -> Result<(), testing::Failure> {
    testing::check!("non-empty".is_empty());
    Ok(())
}
