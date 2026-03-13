// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::mem::MaybeUninit;
use core::slice;

use crate::println;
use crate::shell;

unsafe extern "C" {
    static __data_hark_tests_start: u8;
    static __data_hark_tests_end: u8;
}

static mut TESTS: MaybeUninit<&'static [TestSpec]> = MaybeUninit::uninit();

// Metadata for a kernel test, collected into a linker section.
#[doc(hidden)]
#[repr(C)]
pub struct TestSpec {
    pub suite: &'static str,
    pub case: &'static str,
    pub func: fn() -> Result<(), Failure>,
}

pub(crate) fn init() {
    let tests = unsafe {
        let start: *const TestSpec = (&raw const __data_hark_tests_start).cast();
        debug_assert!(start.is_aligned(), "__data_hark_tests_start misaligned?!");

        let end: *const TestSpec = (&raw const __data_hark_tests_end).cast();
        debug_assert!(end.is_aligned(), "__data_hark_tests_end misaligned?!");

        let len = end.offset_from(start).cast_unsigned();
        slice::from_raw_parts_mut(start.cast_mut(), len)
    };
    sort(tests);
    unsafe {
        (*&raw mut TESTS).write(tests);
    }
}

fn sort(tests: &mut [TestSpec]) {
    for i in 1..tests.len() {
        let mut j = i;
        while j > 0 && (tests[j - 1].suite, tests[j - 1].case) > (tests[j].suite, tests[j].case) {
            tests.swap(j - 1, j);
            j -= 1;
        }
    }
}

fn tests() -> &'static [TestSpec] {
    unsafe { (*&raw const TESTS).assume_init() }
}

/// Describes a test failure with source location and expression.
pub struct Failure {
    pub msg: &'static str,
    pub file: &'static str,
    pub line: u32,
}

/// Asserts that a condition is true, returning a [`Failure`] on false.
#[doc(hidden)]
#[macro_export]
macro_rules! check {
    ($cond:expr) => {
        if !$cond {
            return Err(::hark::testing::Failure {
                file: file!(),
                line: line!(),
                msg: stringify!($cond),
            });
        }
    };
}

#[doc(inline)]
pub use check;

/// Asserts that two expressions are equal, returning a [`Failure`] if not.
#[doc(hidden)]
#[macro_export]
macro_rules! check_eq {
    ($left:expr, $right:expr) => {
        if $left != $right {
            return Err(::hark::testing::Failure {
                file: file!(),
                line: line!(),
                msg: concat!(stringify!($left), " == ", stringify!($right)),
            });
        }
    };
}

#[doc(inline)]
pub use check_eq;

/// test [<suite>]
///
/// Runs tests. If a suite filter is provided, only tests with that
/// suite name run; if no filter is passed, all tests will be run.
#[shell::command(help = "Run kernel tests")]
fn test(mut args: shell::Args) -> bool {
    let suite = args.next().map(|arg| arg.as_str());

    // Up to one argument.
    if args.next().is_some() {
        return false;
    }

    let tests = tests();
    let mut total = 0;
    for t in tests {
        if let Some(suite) = suite
            && t.suite != suite
        {
            continue;
        }
        total += 1;
    }

    println!("\nRunning {total} tests");
    let mut passed = 0u32;
    let mut failed = 0u32;
    for t in tests {
        if let Some(suite) = suite
            && t.suite != suite
        {
            continue;
        }

        crate::print!("{}::{} ... ", t.suite, t.case);
        match (t.func)() {
            Ok(()) => {
                println!("ok");
                passed += 1;
            }
            Err(failure) => {
                println!("FAILED");
                println!("    {}:{}: {}", failure.file, failure.line, failure.msg);
                failed += 1;
            }
        }
    }
    let result = if failed > 0 { "FAILED" } else { "ok" };
    println!("\ntest result: {result}. {passed} passed; {failed} failed");
    true
}
