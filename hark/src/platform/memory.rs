// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::ptr;

use talc::locking::AssumeUnlockable;
use talc::{ErrOnOom, Span, Talc, Talck};

use crate::println;

unsafe extern "C" {
    static __executable_start: u8;
    static _end: u8;
}

// TODO: `AssumeUnlockable` assumes we are a single core system and that no
// allocation occurs within interrupt handlers. Revisit once one of those
// properties no longer holds.
#[global_allocator]
static ALLOCATOR: Talck<AssumeUnlockable, ErrOnOom> = Talc::new(ErrOnOom).lock();

#[derive(Clone, Copy)]
pub struct Range {
    pub start: usize,
    pub size: usize,
}

impl Range {
    const fn end(self) -> usize {
        self.start + self.size
    }
}

pub fn init(memory: &[Range]) {
    let image_start = (&raw const __executable_start).addr();
    let image_end = (&raw const _end).addr();

    println!("Init: Hark loaded at [{image_start:#x}, {image_end:#x})");

    macro_rules! claim {
        ($talc:expr, $start:expr, $size:expr) => {
            println!("Heap: claimed [{:#x}, {:#x})", $start, $start + $size);
            unsafe {
                $talc
                    .claim(Span::from_base_size(
                        ptr::without_provenance_mut($start),
                        $size,
                    ))
                    .unwrap();
            }
        };
    }

    // While it is a safe assumption that we were loaded at the beginning of
    // RAM, we check for memory before the load image just in case.
    let mut talc = ALLOCATOR.lock();
    for range in memory {
        // Range lies strictly before or after the image.
        if range.end() < image_start || image_end < range.start {
            claim!(talc, range.start, range.size);
            continue;
        }

        // Else, we have an overlap. Check for both front and back over-hangs.
        if range.start < image_start {
            claim!(talc, range.start, image_start - range.start);
        }
        if image_end < range.end() {
            claim!(talc, image_end, range.end() - image_end);
        }
    }
}
