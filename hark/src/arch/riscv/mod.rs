// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

mod sbi;

#[allow(unused)]
pub(crate) use sbi::*;

mod exception;

mod start;

#[cfg(not(riscv_m_mode))]
use start::BOOT_HART_ID;

use core::{fmt, mem, ptr};

use crate::arch::ArchCommon;
use crate::println;

use regio::Register as _;

cfg_if::cfg_if! {
    if #[cfg(riscv_m_mode)] {
        use libarch::riscv::{Marchid, Mhartid, Misa, Mvendorid, Mimpid};
        use crate::print;
    }
}

#[cfg(riscv_m_mode)]
type Xscratch = libarch::riscv::Mscratch;

#[cfg(not(riscv_m_mode))]
type Xscratch = libarch::riscv::Sscratch;

#[used]
static mut PERCPU: [PerCpu; 1] = [const { unsafe { mem::zeroed() } }; 1];

#[repr(C)]
#[derive(Debug, Default)]
pub struct PerCpu {
    // The scratch area used at the top of the main exception handling routine
    // to store register state for later exception context printing. We keep
    // this first so that the percpu pointer also points to the scratch area,
    // which the exception handling routine can take advantage of.
    pub exception_scratch_regs: Regs,
}

#[repr(C)]
#[derive(Debug, Default)]
pub struct Regs {
    pub pc: usize,
    pub ra: usize,
    pub sp: usize,
    pub gp: usize,
    pub tp: usize,
    pub t0: usize,
    pub t1: usize,
    pub t2: usize,
    pub s0: usize,
    pub s1: usize,
    pub a0: usize,
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub a4: usize,
    pub a5: usize,
    pub a6: usize,
    pub a7: usize,
    pub s2: usize,
    pub s3: usize,
    pub s4: usize,
    pub s5: usize,
    pub s6: usize,
    pub s7: usize,
    pub s8: usize,
    pub s9: usize,
    pub s10: usize,
    pub s11: usize,
    pub t3: usize,
    pub t4: usize,
    pub t5: usize,
    pub t6: usize,
}

impl fmt::Display for Regs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Width: "0x".len() + (2 hex digits / byte) * (size_of::<usize> bytes)
        let w = 2 + 2 * size_of::<usize>();
        writeln!(
            f,
            "  pc:  {:#0w$x}  ra:  {:#0w$x}  sp:  {:#0w$x}  gp:  {:#0w$x} ",
            self.pc, self.ra, self.sp, self.gp
        )?;
        writeln!(
            f,
            "  tp:  {:#0w$x}  t0:  {:#0w$x}  t1:  {:#0w$x}  t2:  {:#0w$x} ",
            self.tp, self.t0, self.t1, self.t2
        )?;
        writeln!(
            f,
            "  s0:  {:#0w$x}  s1:  {:#0w$x}  a0:  {:#0w$x}  a1:  {:#0w$x} ",
            self.s0, self.s1, self.a0, self.a1
        )?;
        writeln!(
            f,
            "  a2:  {:#0w$x}  a3:  {:#0w$x}  a4:  {:#0w$x}  a5:  {:#0w$x} ",
            self.a2, self.a3, self.a4, self.a5
        )?;
        writeln!(
            f,
            "  a6:  {:#0w$x}  a7:  {:#0w$x}  s2:  {:#0w$x}  s3:  {:#0w$x} ",
            self.a6, self.a7, self.s2, self.s3
        )?;
        writeln!(
            f,
            "  s4:  {:#0w$x}  s5:  {:#0w$x}  s6:  {:#0w$x}  s7:  {:#0w$x} ",
            self.s4, self.s5, self.s6, self.s7
        )?;
        writeln!(
            f,
            "  s8:  {:#0w$x}  s9:  {:#0w$x}  s10: {:#0w$x}  s11: {:#0w$x} ",
            self.s8, self.s9, self.s10, self.s11
        )?;
        writeln!(
            f,
            "  t3:  {:#0w$x}  t4:  {:#0w$x}  t5:  {:#0w$x}  t6:  {:#0w$x} ",
            self.t3, self.t4, self.t5, self.t6
        )
    }
}

pub(super) fn get_percpu() -> &'static PerCpu {
    // TODO: SMP support.
    unsafe { &*&raw const PERCPU[0] }
}

pub(super) struct Arch {}

impl ArchCommon for Arch {
    #[inline]
    fn init() {
        let percpu = get_percpu();
        Xscratch::from(ptr::from_ref(percpu).addr()).write();
        exception::init();
    }

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
        println!("Boot hart ID: {BOOT_HART_ID:#}");
    }
}
