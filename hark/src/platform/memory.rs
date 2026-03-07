// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use crate::println;

unsafe extern "C" {
    static __executable_start: u8;
    static _end: u8;
}

#[allow(unused)]
#[derive(Clone, Copy)]
pub struct Range {
    pub start: usize,
    pub size: usize,
}

#[allow(unused)]
impl Range {
    const fn end(self) -> usize {
        self.start + self.size
    }
}

pub fn init(_memory: &[Range]) {
    let image_start = (&raw const __executable_start).addr();
    let image_end = (&raw const _end).addr();

    println!("Init: Hark loaded at [{image_start:#x}, {image_end:#x})");
}
