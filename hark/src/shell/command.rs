// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::cmp::max;
use core::mem::MaybeUninit;
use core::ops::{Add, Mul};
use core::slice;
use core::str::SplitAsciiWhitespace;

use super::command;
use crate::platform::console;

unsafe extern "C" {
    static __data_hark_commands_start: u8;
    static __data_hark_commands_end: u8;
}

static mut COMMANDS: MaybeUninit<&'static [CommandSpec]> = MaybeUninit::uninit();

// Collects the command specs from the linker section and sorts them by name.
pub(super) fn init() {
    let commands = unsafe {
        let start: *const CommandSpec = (&raw const __data_hark_commands_start).cast();
        debug_assert!(
            start.is_aligned(),
            "__data_hark_commands_start misaligned?!"
        );

        let end: *const CommandSpec = (&raw const __data_hark_commands_end).cast();
        debug_assert!(end.is_aligned(), "__data_hark_commands_end misaligned?!");

        let len = end.offset_from(start).cast_unsigned();
        slice::from_raw_parts_mut(start.cast_mut(), len)
    };
    sort(commands);
    unsafe {
        (*&raw mut COMMANDS).write(commands);
    }
}

// A slice's own sort method brings in a *lot* of code and insertion sort should
// be totally fine for the expected number of commands.
fn sort(commands: &mut [CommandSpec]) {
    for i in 1..commands.len() {
        let mut j = i;
        while j > 0 && commands[j - 1].name > commands[j].name {
            commands.swap(j - 1, j);
            j -= 1;
        }
    }
}

fn commands() -> &'static [CommandSpec] {
    unsafe { (*&raw const COMMANDS).assume_init() }
}

// Provides metadata for a shell command, implicitly defined by
// #[shell_command] and collected into a special linker section
// (.data.hark.commands).
#[doc(hidden)]
#[repr(C)]
pub struct CommandSpec {
    pub name: &'static str,
    pub desc: &'static str,
    pub help: &'static str,
    pub func: fn(args: Args) -> bool,
}

/// Represents a hark shell command line, or the tail of one.
#[derive(Clone)]
pub struct Args<'a>(SplitAsciiWhitespace<'a>);

impl<'a> Iterator for Args<'a> {
    type Item = Arg<'a>;

    fn next(&mut self) -> Option<Arg<'a>> {
        self.0.next().map(Arg)
    }
}

/// An argument in a hark shell command.
pub struct Arg<'a>(&'a str);

impl<'a> Arg<'a> {
    /// Returns the raw, ASCII string value of the argument.
    pub fn as_str(&self) -> &'a str {
        self.0
    }

    /// Parses the argument as a decimal integer (for any integral type).
    /// Returns `None` on invalid input.
    pub fn as_decimal<Int>(&self) -> Option<Int>
    where
        Int: From<u8> + Add<Output = Int> + Mul<Output = Int> + sealed::Signedness,
    {
        let mut bytes = self.0.as_bytes();
        // First account for sign.
        let mut negate = false;
        if bytes[0] == b'-' {
            if !Int::SIGNED {
                return None;
            }
            negate = true;
            bytes = &bytes[1..];
        }

        let mut val = Int::from(0u8);
        for ch in bytes {
            match ch {
                b'0'..=b'9' => {
                    val = Int::from(10u8) * val + Int::from(ch - b'0');
                }
                _ => return None,
            }
        }
        if negate {
            val = val.negate();
        }
        Some(val)
    }

    /// Parses the argument as a hex integer (with a `0x` prefix, for any
    /// integral type). Returns `None` on invalid input.
    pub fn as_hex<Int>(&self) -> Option<Int>
    where
        Int: From<u8> + Add<Output = Int> + Mul<Output = Int> + sealed::Signedness,
    {
        let mut bytes = self.0.as_bytes();

        // First account for sign.
        let mut negate = false;
        if bytes[0] == b'-' {
            if !Int::SIGNED {
                return None;
            }
            negate = true;
            bytes = &bytes[1..];
        }

        if bytes[0] != b'0' || bytes[1] != b'x' {
            return None;
        }
        bytes = &bytes[2..];
        let mut val = Int::from(0u8);
        for ch in bytes {
            let digit = match ch {
                b'0'..=b'9' => Int::from(ch - b'0'),
                b'a'..=b'f' => Int::from(ch - b'a' + 10),
                _ => return None,
            };
            val = Int::from(16u8) * val + digit;
        }
        if negate {
            val = val.negate();
        }
        Some(val)
    }
}

impl<'a> PartialEq<&'a str> for Arg<'_> {
    fn eq(&self, other: &&'a str) -> bool {
        self.0 == *other
    }
}

// Allows as_decimal/as_hex to handle both signed and unsigned integer
// types without separate methods.
mod sealed {
    use core::ops::Neg;

    pub trait Signedness: Sized {
        const SIGNED: bool = false;
        fn negate(self) -> Self {
            self
        }
    }

    macro_rules! impl_signedness_signed {
        ($signed:ty) => {
            impl Signedness for $signed {
                const SIGNED: bool = true;

                fn negate(self) -> Self {
                    Neg::neg(self)
                }
            }
        };
    }
    impl_signedness_signed!(i8);
    impl_signedness_signed!(i16);
    impl_signedness_signed!(i32);
    impl_signedness_signed!(i64);
    impl_signedness_signed!(isize);
}

// Looks up and executes the named command, or prints an error.
pub(super) fn dispatch(command: &str) {
    let mut args = Args(command.split_ascii_whitespace());
    let Some(name) = args.next() else {
        return;
    };

    // TODO: binary search
    for cmd in commands() {
        if name == cmd.name {
            if !(cmd.func)(args) {
                console::write(b"\nInvalid args. Seek help:\n\n");
                console::write(cmd.desc.as_bytes());
                console::write_byte(b'\n');
            }
            return;
        }
    }
    console::write(b"Unknown command: `");
    console::write(name.as_str().as_bytes());
    console::write(b"`; run `help` to list available commands\n");
}

/// help [<command>]
///
/// Prints the command's full description if a command is provided, or else
/// lists all available commands.
#[command(help = "List all commands")]
fn help(mut args: Args) -> bool {
    if let Some(name) = args.next() {
        // TODO: binary search
        for cmd in commands() {
            if name == cmd.name {
                console::write(cmd.desc.as_bytes());
                console::write_byte(b'\n');
                return true;
            }
        }
    }

    list_commands();
    true
}

fn list_commands() {
    let mut longest = 0;
    for cmd in commands() {
        longest = max(longest, cmd.name.len());
    }

    let pad = |n| {
        for _ in 0..n {
            console::write_byte(b' ');
        }
    };

    console::write(b"Commands:\n");
    for cmd in commands() {
        console::write(b"* ");
        console::write(cmd.name.as_bytes());
        pad(longest - cmd.name.len() + 4);
        console::write(cmd.help.as_bytes());
        console::write_byte(b'\n');
    }
}
