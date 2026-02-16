// Copyright (c) 2025 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

mod sbi;

#[allow(unused)]
pub(crate) use sbi::*;

mod start;

use crate::arch::ArchCommon;
use crate::println;

cfg_if::cfg_if! {
    if #[cfg(riscv_m_mode)] {
        use libarch::riscv::{Marchid, Mhartid, Misa, Mvendorid, Mimpid};
        use regio::Register as _;
        use crate::print;
    }
}

pub(super) struct Arch {}

impl ArchCommon for Arch {
    #[inline]
    #[cfg(riscv_m_mode)]
    fn print_machine_context() {
        println!("Entry mode: M");
        println!("Boot hart ID: {:#}", *Mhartid::read());
        println!(
            "mvendorid, marchid, mimpid: {:#}, {:#}, {:#}",
            *Mvendorid::read(),
            *Marchid::read(),
            *Mimpid::read()
        );
        print!("misa: ");
        let mut first = true;
        for (metadata, value) in Misa::read().iter().rev() {
            if value == 0 {
                continue;
            }
            if !first {
                print!(",");
            }
            print!("{}", metadata.name);
            first = false;
        }
        print!("\n");
    }

    #[inline]
    #[cfg(not(riscv_m_mode))]
    fn print_machine_context() {
        println!("Entry mode: S");
    }
}
