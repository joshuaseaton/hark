// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

#![no_std]
#![no_main]

use hark::println;

// TODO: do much more.
#[unsafe(no_mangle)]
extern "Rust" fn hark_app_main() {
    println!("Example: Hello from a Hark app!");
}
