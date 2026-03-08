// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use crate::println;

unsafe extern "C" {
    static __boot_flash_start: u8;
    static __boot_flash_end: u8;
    static __boot_ram_start: u8;
    static __boot_ram_end: u8;
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
    let flash_start = (&raw const __boot_flash_start).addr();
    let flash_end = (&raw const __boot_flash_end).addr();
    println!("Boot flash: [{flash_start:#x}, {flash_end:#x})");

    let ram_start = (&raw const __boot_ram_start).addr();
    let ram_end = (&raw const __boot_ram_end).addr();
    println!("Boot RAM: [{ram_start:#x}, {ram_end:#x})");
}
