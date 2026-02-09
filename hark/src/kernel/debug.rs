// Copyright (c) 2025 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::{fmt, ops};

use elf::endian::LittleEndian;
use elf::file::Class as ElfClass;
use elf::note::{Note, NoteIterator};

unsafe extern "C" {
    // Boundaries of the .note.gnu.build-id section.
    static __start_note_gnu_build_id: u8;
    static __end_note_gnu_build_id: u8;
}

/// Represents a GNU build ID.
#[derive(Debug)]
pub struct BuildId(&'static [u8]);

impl ops::Deref for BuildId {
    type Target = &'static [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for BuildId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x")?;
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        writeln!(f)
    }
}

/// Returns the kernel's encoded GNU build ID.
///
/// # Panics
///
/// This will panic if the .note.gnu.build-id section contains a malformed GNU
/// build ID note.
pub fn build_id() -> BuildId {
    const CLASS: ElfClass = if cfg!(target_pointer_width = "32") {
        ElfClass::ELF32
    } else {
        ElfClass::ELF64
    };
    let build_id_start = &raw const __start_note_gnu_build_id;
    let build_id_end = &raw const __end_note_gnu_build_id;
    let build_id_slice = unsafe {
        core::slice::from_raw_parts(
            build_id_start,
            build_id_end.offset_from_unsigned(build_id_start),
        )
    };
    let mut iter = NoteIterator::new(LittleEndian {}, CLASS, 1, build_id_slice);
    let note = iter
        .next()
        .expect(".note.gnu.build-id did not contain an ELF note?!");
    let Note::GnuBuildId(build_id) = note else {
        panic!(".note.gnu.build-id did not contain a build ID?!");
    };
    BuildId(build_id.0)
}
