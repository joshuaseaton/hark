// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::arch::naked_asm;
use core::mem::offset_of;

use libarch::riscv::csr::Mstatus;

use super::{load, store};
use crate::thread::thread_exit;

// Call-preserved registers for context switching.
#[repr(C)]
#[derive(Debug)]
pub struct Context {
    ra: usize,
    sp: usize,
    s0: usize,
    s1: usize,
    s2: usize,
    s3: usize,
    s4: usize,
    s5: usize,
    s6: usize,
    s7: usize,
    s8: usize,
    s9: usize,
    s10: usize,
    s11: usize,
}

impl Context {
    pub const fn zero() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s0: 0,
            s1: 0,
            s2: 0,
            s3: 0,
            s4: 0,
            s5: 0,
            s6: 0,
            s7: 0,
            s8: 0,
            s9: 0,
            s10: 0,
            s11: 0,
        }
    }

    // Creates an initial context for a new thread.
    //
    // After switching into this context, the thread will ret into `trampoline`,
    // which is responsible for calling `entry` and handling a thread exit.
    pub fn new(sp: usize, entry: usize, arg: usize) -> Self {
        Self {
            ra: trampoline as *const () as usize,
            sp,
            s0: 0,     // Bottom of callstack
            s1: entry, // Expected by trampoline
            s2: arg,   // Expected by trampoline
            ..Self::zero()
        }
    }

    pub unsafe fn switch(&mut self, new: &Context) {
        unsafe {
            switch_contexts(self, new);
        }
    }
}

// trampoline expects the entry function at s1 and an argument at s2.
#[unsafe(naked)]
unsafe extern "C" fn trampoline() {
    naked_asm!(
        // We may be jumping onto the trampoline from an interrupt context, in
        // which case this the opportunity to re-enable interrupts.
        "csrsi mstatus, {mstatus_mie_mask}",
        "mv a0, s2",
        "jalr s1",
        // If we return, then we tail into the thread exit routine.
        "tail {thread_exit}",
        mstatus_mie_mask = const Mstatus::MIE_MASK,
        thread_exit = sym thread_exit,
    );
}

// Saves call-preserved registers into `old` and restores them from `new`,
// effectively switching execution contexts.
#[unsafe(naked)]
unsafe extern "C" fn switch_contexts(old: &mut Context, new: &Context) {
    naked_asm!(
        // Save call-preserved registers into `old` (a0).
        store!("ra, {ra}(a0)"),
        store!("sp, {sp}(a0)"),
        store!("s0, {s0}(a0)"),
        store!("s1, {s1}(a0)"),
        store!("s2, {s2}(a0)"),
        store!("s3, {s3}(a0)"),
        store!("s4, {s4}(a0)"),
        store!("s5, {s5}(a0)"),
        store!("s6, {s6}(a0)"),
        store!("s7, {s7}(a0)"),
        store!("s8, {s8}(a0)"),
        store!("s9, {s9}(a0)"),
        store!("s10, {s10}(a0)"),
        store!("s11, {s11}(a0)"),

        // Restore call-preserved registers from `new` (a1).
        load!("ra, {ra}(a1)"),
        load!("sp, {sp}(a1)"),
        load!("s0, {s0}(a1)"),
        load!("s1, {s1}(a1)"),
        load!("s2, {s2}(a1)"),
        load!("s3, {s3}(a1)"),
        load!("s4, {s4}(a1)"),
        load!("s5, {s5}(a1)"),
        load!("s6, {s6}(a1)"),
        load!("s7, {s7}(a1)"),
        load!("s8, {s8}(a1)"),
        load!("s9, {s9}(a1)"),
        load!("s10, {s10}(a1)"),
        load!("s11, {s11}(a1)"),
        "ret",
        ra = const offset_of!(Context, ra),
        sp = const offset_of!(Context, sp),
        s0 = const offset_of!(Context, s0),
        s1 = const offset_of!(Context, s1),
        s2 = const offset_of!(Context, s2),
        s3 = const offset_of!(Context, s3),
        s4 = const offset_of!(Context, s4),
        s5 = const offset_of!(Context, s5),
        s6 = const offset_of!(Context, s6),
        s7 = const offset_of!(Context, s7),
        s8 = const offset_of!(Context, s8),
        s9 = const offset_of!(Context, s9),
        s10 = const offset_of!(Context, s10),
        s11 = const offset_of!(Context, s11),
    )
}
