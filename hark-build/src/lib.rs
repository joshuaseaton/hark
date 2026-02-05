// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use std::env;

// An invalid character on the linker command line, so a convenient separator to
// use for packaging a list of arguments into a single string.
const LINK_ARG_SEPARATOR: &str = "|";

// Intended to be called only in Hark's own build script, this will emit the
// link arg metadata intended to be applied by a dependent Hark system via
// `declare_system()`.
#[doc(hidden)]
pub fn emit_metadata_for_system(link_args: &[String]) {
    println!(
        "cargo::metadata=LINK_ARGS={}",
        link_args.join(LINK_ARG_SEPARATOR)
    );
}

/// Intended to be called in a Hark system build script, this will emit the
/// appropriate link arguments.
///
/// # Panics
///
/// Panics if the would-be Hark system does not actually depend on the hark
/// crate.
pub fn declare_system() {
    let concatenated_link_args = env::var("DEP_HARK_LINK_ARGS")
        .expect("$DEP_HARK_LINK_ARGS not defined. Missing a dependency on the hark crate!");
    let link_args: Vec<&str> = concatenated_link_args.split(LINK_ARG_SEPARATOR).collect();
    for arg in link_args {
        println!("cargo::rustc-link-arg={arg}");
    }
}
