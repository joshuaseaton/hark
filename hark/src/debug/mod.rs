// Copyright (c) 2025 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::fmt;
use core::mem::MaybeUninit;

use derive_more::{Deref, From};
use libarch::Backtrace;

use crate::{ConsoleWitness, println};

unsafe extern "C" {
    static __boot_flash_start: u8;
    static __data_lma_start: u8;
    static __boot_ram_start: u8;
    static __boot_ram_end: u8;

    // Boundaries of the build ID within the .note.gnu.build-id section.
    static __build_id_start: u8;
    static __note_gnu_build_id_end: u8;
}

// The kernel's GNU build ID. Despite being `mut` this will be set once while
// the kernel is single threaded in init_build_id(), after which it may be
// freely accessed as immutable via build_id().
static mut BUILD_ID: MaybeUninit<BuildId> = MaybeUninit::uninit();

/// Represents a GNU build ID.
#[derive(Clone, Copy, Debug, Deref, Eq, From, PartialEq)]
pub struct BuildId(&'static [u8]);

impl fmt::Display for BuildId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        Ok(())
    }
}

pub(crate) fn init(_: &ConsoleWitness) {
    init_build_id();
}

fn init_build_id() {
    let build_id_start = &raw const __build_id_start;
    let build_id_end = &raw const __note_gnu_build_id_end;
    let build_id = unsafe {
        core::slice::from_raw_parts(
            build_id_start,
            build_id_end.offset_from_unsigned(build_id_start),
        )
    };
    // Unsafe: Setting it once while we are single-threaded.
    unsafe {
        (*&raw mut BUILD_ID).write(BuildId(build_id));
    }
}

/// Returns the kernel's encoded GNU build ID.
///
/// # Panics
///
/// This will panic if the .note.gnu.build-id section contains a malformed GNU
/// build ID note.
pub fn build_id() -> BuildId {
    // Safety: This should only be accessed after init_build_id() and is
    // read-only.
    unsafe { BUILD_ID.assume_init() }
}

// Prints the LLVM symbolizer markup for offline symbolization.
pub(crate) fn print_backtrace(fp: usize, pc: Option<usize>) {
    println!("Backtrace:");
    print_reset_element();
    print_module_element();
    print_mmap_element();

    let idx_offset = if let Some(pc) = pc {
        print_bt_pc_element(0, pc);
        1
    } else {
        0
    };

    for (idx, addr) in Backtrace::new(fp).enumerate() {
        print_bt_ra_element(idx + idx_offset, addr);
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
    // This may be called before the build ID was parsed, and we may even be
    // mid-panic due to a malformed build ID.
    let build_id = build_id();
    if !build_id.is_empty() {
        println!("{{{{{{module:0:hark:elf:{}}}}}}}", build_id);
    }
}

#[inline(never)]
fn print_mmap_element() {
    let flash_start = &raw const __boot_flash_start;
    let flash_nonwritable_end = &raw const __data_lma_start;
    let flash_nonwritable_len = unsafe { flash_nonwritable_end.offset_from_unsigned(flash_start) };
    println!(
        "{{{{{{mmap:{flash_start:#?}:{flash_nonwritable_len:#x}:load:0:rx:{flash_start:#?}}}}}}}"
    );

    let ram_start = &raw const __boot_ram_start;
    let ram_end = &raw const __boot_ram_end;
    let ram_len = unsafe { ram_end.offset_from_unsigned(ram_start) };
    println!("{{{{{{mmap:{ram_start:#?}:{ram_len:#x}:load:0:rw:{ram_start:#?}}}}}}}");
}

#[inline(never)]
fn print_bt_ra_element(index: usize, return_address: usize) {
    println!("{{{{{{bt:{index}:{:#x}:ra}}}}}}", return_address);
}

#[inline(never)]
fn print_bt_pc_element(index: usize, pc: usize) {
    println!("{{{{{{bt:{index}:{:#x}:pc}}}}}}", pc);
}
