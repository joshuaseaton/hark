// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::arch::naked_asm;
use core::mem::offset_of;

use libarch::riscv::{ExceptionCode, TrapVectorMode};
use regio::Register;

use crate::arch::riscv::{PerCpu, Regs, get_percpu};
use crate::kernel::panic_common;
use crate::{print, println};

cfg_if::cfg_if! {
    if #[cfg(riscv_m_mode)] {
        type Xcause = libarch::riscv::Mcause;
        type Xtval = libarch::riscv::Mtval;
        type Xtvec = libarch::riscv::Mtvec;

        macro_rules! swap_xscratch_with_t0 {
            () => {
                "csrrw t0, mscratch, t0"
            };
        }
        macro_rules! read_xepc_into_t0 {
            () => {
                "csrr t0, mepc"
            };
        }

    } else {
        type Xcause = libarch::riscv::Scause;
        type Xtval = libarch::riscv::Stval;
        type Xtvec = libarch::riscv::Stvec;

        macro_rules! swap_xscratch_with_t0 {
            () => {
                "csrrw t0, sscratch, t0"
            };
        }
        macro_rules! read_xepc_into_t0 {
            () => {
                "csrr t0, sepc"
            };
        }

    }
}

// Relied upon at the top of exception_entry().
const _: () = const { assert!(offset_of!(PerCpu, exception_scratch_regs) == 0) };

pub(super) fn init() {
    let entry = (exception_entry as *const ()).addr();
    Xtvec::from(entry) // TODO... unshifted!
        .set_mode(TrapVectorMode::Direct)
        .write();
}

#[unsafe(no_mangle)]
#[unsafe(naked)]
extern "C" fn exception_entry() {
    naked_asm!(
        // Reading in xscratch to get the percpu pointer will trash a register
        // that we have yet to record. So we swap the xscratch value with t0,
        // saving recording t0 for later when we have a spare register. Note
        // that the register scratch area is the first field in the percpu
        // struct (asserted above) so the pointer points to that as well.
        swap_xscratch_with_t0!(),
        r#"
        // Save pc for the end for when we have a spare register for reading in
        // xepc.
        sw ra, {ra_offset}(t0)
        sw sp, {sp_offset}(t0)
        sw gp, {gp_offset}(t0)
        sw tp, {tp_offset}(t0)
        sw t1, {t1_offset}(t0)
        sw t2, {t2_offset}(t0)
        sw s0, {s0_offset}(t0)
        sw s1, {s1_offset}(t0)
        sw a0, {a0_offset}(t0)
        sw a1, {a1_offset}(t0)
        sw a2, {a2_offset}(t0)
        sw a3, {a3_offset}(t0)
        sw a4, {a4_offset}(t0)
        sw a5, {a5_offset}(t0)
        sw a6, {a6_offset}(t0)
        sw a7, {a7_offset}(t0)
        sw s2, {s2_offset}(t0)
        sw s3, {s3_offset}(t0)
        sw s4, {s4_offset}(t0)
        sw s5, {s5_offset}(t0)
        sw s6, {s6_offset}(t0)
        sw s7, {s7_offset}(t0)
        sw s8, {s8_offset}(t0)
        sw s9, {s9_offset}(t0)
        sw s10, {s10_offset}(t0)
        sw s11, {s11_offset}(t0)
        sw t3, {t3_offset}(t0)
        sw t4, {t4_offset}(t0)
        sw t5, {t5_offset}(t0)
        sw t6, {t6_offset}(t0)
        "#,
        // Before we swap t0 back with the xscratch pointer, we save the pointer
        // since we still have t0 and pc to record.
        "mv t1, t0",
        swap_xscratch_with_t0!(),
        "sw t0, {t0_offset}(t1)",
        read_xepc_into_t0!(),
        r#"
        sw t0, {pc_offset}(t1)

        // All registers recorded! Back to Rust...
        j handle_exception
        "#,
        pc_offset = const offset_of!(Regs, pc),
        ra_offset = const offset_of!(Regs, ra),
        sp_offset = const offset_of!(Regs, sp),
        gp_offset = const offset_of!(Regs, gp),
        tp_offset = const offset_of!(Regs, tp),
        t0_offset = const offset_of!(Regs, t0),
        t1_offset = const offset_of!(Regs, t1),
        t2_offset = const offset_of!(Regs, t2),
        s0_offset = const offset_of!(Regs, s0),
        s1_offset = const offset_of!(Regs, s1),
        a0_offset = const offset_of!(Regs, a0),
        a1_offset = const offset_of!(Regs, a1),
        a2_offset = const offset_of!(Regs, a2),
        a3_offset = const offset_of!(Regs, a3),
        a4_offset = const offset_of!(Regs, a4),
        a5_offset = const offset_of!(Regs, a5),
        a6_offset = const offset_of!(Regs, a6),
        a7_offset = const offset_of!(Regs, a7),
        s2_offset = const offset_of!(Regs, s2),
        s3_offset = const offset_of!(Regs, s3),
        s4_offset = const offset_of!(Regs, s4),
        s5_offset = const offset_of!(Regs, s5),
        s6_offset = const offset_of!(Regs, s6),
        s7_offset = const offset_of!(Regs, s7),
        s8_offset = const offset_of!(Regs, s8),
        s9_offset = const offset_of!(Regs, s9),
        s10_offset = const offset_of!(Regs, s10),
        s11_offset = const offset_of!(Regs, s11),
        t3_offset = const offset_of!(Regs, t3),
        t4_offset = const offset_of!(Regs, t4),
        t5_offset = const offset_of!(Regs, t5),
        t6_offset = const offset_of!(Regs, t6),
    )
}

#[unsafe(no_mangle)]
extern "C" fn handle_exception() -> ! {
    let regs = &get_percpu().exception_scratch_regs;
    panic_common(regs.s0, Some(regs.pc), || {
        let xtval = *Xtval::read();
        print!("Exception: {}", Xcause::read().exception_code());
        match Xcause::read().exception_code() {
            // In these cases xtval holds the associated address.
            ExceptionCode::INSTRUCTION_ADDRESS_MISALIGNED
            | ExceptionCode::INSTRUCTION_ACCESS_FAULT
            | ExceptionCode::BREAKPOINT
            | ExceptionCode::LOAD_ADDRESS_MISALIGNED
            | ExceptionCode::STORE_OR_AMO_ADDRESS_ACCESS_FAULT
            | ExceptionCode::STORE_OR_AMO_ADDRESS_MISALIGNED
            | ExceptionCode::INSTRUCTION_PAGE_FAULT
            | ExceptionCode::LOAD_PAGE_FAULT
            | ExceptionCode::STORE_OR_AMO_PAGE_FAULT => {
                println!(" (@ {xtval:#x})");
            }
            _ => {
                if xtval != 0 {
                    println!(" ({xtval:#x})");
                }
            }
        }

        println!("Registers:\n{}", regs);
    });
}
