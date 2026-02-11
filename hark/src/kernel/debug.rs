// Copyright (c) 2025 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::{fmt, ops, ptr};

use libarch::Backtrace;
use zerocopy::{FromBytes, Immutable, KnownLayout};

use crate::{print, println};

unsafe extern "C" {
    static __executable_start: u8;
    static _end: u8;

    // Boundaries of the .note.gnu.build-id section.
    static __note_gnu_build_id_start: u8;
    static __note_gnu_build_id_end: u8;
}

// The kernel's GNU build ID. Despite being `mut` this will be set once while
// the kernel is single threaded in init_build_id(), after which it may be
// freely accessed as immutable via build_id().
static mut BUILD_ID: BuildId = BuildId(&[]);

/// Represents a GNU build ID.
#[derive(Clone, Copy, Debug)]
pub struct BuildId(&'static [u8]);

impl ops::Deref for BuildId {
    type Target = &'static [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for BuildId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

// Elf32_Nhdr or Elf64_Nhdr.
#[derive(FromBytes, Immutable, KnownLayout)]
#[repr(C)]
struct ElfNhdr {
    namesz: u32,
    descsz: u32,
    type_: u32,
}

pub(crate) fn early_init() {
    init_build_id();
}

fn init_build_id() {
    const NT_GNU_BUILD_ID: u32 = 3;
    const GNU_NOTE_NAME: &[u8; 4] = b"GNU\0";

    let build_id_start = &raw const __note_gnu_build_id_start;
    let build_id_end = &raw const __note_gnu_build_id_end;
    let build_id_slice = unsafe {
        core::slice::from_raw_parts(
            build_id_start,
            build_id_end.offset_from_unsigned(build_id_start),
        )
    };

    let (
        ElfNhdr {
            namesz,
            descsz,
            type_,
        },
        rest,
    ) = ElfNhdr::read_from_prefix(build_id_slice)
        .expect(".note.gnu.build-id too small for note header");
    assert_eq!(
        type_, NT_GNU_BUILD_ID,
        ".note.gnu.build-id has type {type_} != NT_GNU_BUILD_ID",
    );

    let (name, rest) = rest
        .split_at_checked(namesz as usize)
        .expect(".note.gnu.build-id malformed: namesz exceeds the end of the note");
    assert_eq!(
        name, GNU_NOTE_NAME,
        ".note.gnu.build-id has name {name:#?} != {GNU_NOTE_NAME:#?}"
    );
    let (build_id, _) = rest
        .split_at_checked(descsz as usize)
        .expect(".note.gnu.build-id malformed: descsz exceeds the end of the note");

    // Unsafe: Setting it once while we are single-threaded.
    unsafe {
        ptr::write_volatile(&raw mut BUILD_ID, BuildId(build_id));
    }
}

/// Returns the kernel's encoded GNU build ID.
///
/// # Panics
///
/// This will panic if the .note.gnu.build-id section contains a malformed GNU
/// build ID note.
pub fn build_id() -> BuildId {
    // Saftey: This should only be accessed after init_build_id() and is
    // read-only.
    unsafe { ptr::read_volatile(&raw const BUILD_ID) }
}

// Prints the LLVM symbolizer markup for offline symbolization.
#[inline(always)]
pub(crate) fn print_backtrace() {
    println!("Backtrace:");
    print_reset_element();
    print_module_element();
    print_mmap_element();
    for (idx, addr) in Backtrace::new().enumerate() {
        print_bt_element(idx, addr);
    }
}

//
// Printing is stack-hungry, so we put these print routines in inline(never)
// wrappers to avoid stack overflows.
//

#[inline(never)]
fn print_reset_element() {
    println!("{{{{{{reset}}}}}}");
}

#[inline(never)]
fn print_module_element() {
    // This may be called before the build ID was parsed, and maybe are even
    // mid-panic due to a malformed build ID.
    let build_id = build_id();
    if !build_id.is_empty() {
        println!("{{{{{{module:0:hark:elf:{}}}}}}}}}", build_id);
    }
}

#[inline(never)]
fn print_mmap_element() {
    let executable_start = &raw const __executable_start;
    let executable_end = &raw const _end;
    let executable_len = unsafe { executable_end.offset_from_unsigned(executable_start) };
    println!(
        "{{{{{{mmap:{executable_start:#?}:{executable_len:#x}:load:0:rwx:{executable_start:#?}}}}}}}"
    );
}

#[inline(never)]
fn print_bt_element(index: usize, return_address: usize) {
    println!("{{{{{{bt:{index}:{:#x}:ra}}}}}}", return_address - 1);
}
