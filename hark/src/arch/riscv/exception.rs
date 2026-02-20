// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::arch::naked_asm;
use core::mem::offset_of;

use libarch::riscv::{ExceptionCode, InterruptCode, TrapVectorMode};
use regio::Register as _;

use crate::arch::riscv::{Regs, Xie, Xstatus, Xtval, Xtvec};
use crate::kernel::panic_common;
use crate::{print, println};

cfg_if::cfg_if! {
    if #[cfg(not(riscv_m_mode))] {
        use libarch::riscv::Stimecmp;
    }
}

cfg_if::cfg_if! {
    if #[cfg(riscv_m_mode)] {
        macro_rules! read_xcause_into_a0 { () => { "csrr a0, mcause" }; }
        macro_rules! read_xepc_into_t0 { () => { "csrr t0, mepc" }; }
        macro_rules! set_xie_fn { () => { |reg: &mut Xstatus| { reg.set_mie(true); } }; }
        macro_rules! xret { () => { "mret" }; }
    } else {
        macro_rules! read_xcause_into_a0 { () => { "csrr a0, scause" }; }
        macro_rules! read_xepc_into_t0 { () => { "csrr t0, sepc" }; }
        macro_rules! set_xie_fn { () => { |reg: &mut Xstatus| { reg.set_sie(true); } }; }
        macro_rules! xret { () => { "sret" }; }
    }
}

cfg_if::cfg_if! {
    if #[cfg(target_pointer_width = "32")] {
        macro_rules! store { ($args:literal) => { concat!("sw ", $args) }; }
        macro_rules! load { ($args:literal) => { concat!("lw ", $args) }; }
    } else {
        macro_rules! store { ($args:literal) => { concat!("sd ", $args) }; }
        macro_rules! load { ($args:literal) => { concat!("ld ", $args) }; }
    }
}

pub(super) fn init() {
    let entry = (exception_entry as *const ()).addr();
    Xtvec::from(entry) // TODO... unshifted!
        .set_mode(TrapVectorMode::Direct)
        .write();

    // Enable all supported interrupts.
    // TODO: support more!
    Xie::from(0).set_stie(true).write();
    Xstatus::modify(set_xie_fn!());
}

#[unsafe(no_mangle)]
#[unsafe(naked)]
extern "C" fn exception_entry() {
    naked_asm!(
        // We save registers onto the stack, making sure not to trash any before
        // we do.
        //
        // Save pc for later, when we have a spare register for reading in xepc.
        "  addi sp, sp, -{sizeof_regs}",
        store!("ra, {ra_offset}(sp)"),
        // We've just modified sp, so defer recording the original as well.
        store!("gp, {gp_offset}(sp)"),
        store!("tp, {tp_offset}(sp)"),
        store!("t0, {t0_offset}(sp)"),
        store!("t1, {t1_offset}(sp)"),
        store!("t2, {t2_offset}(sp)"),
        store!("s0, {s0_offset}(sp)"),
        store!("s1, {s1_offset}(sp)"),
        store!("a0, {a0_offset}(sp)"),
        store!("a1, {a1_offset}(sp)"),
        store!("a2, {a2_offset}(sp)"),
        store!("a3, {a3_offset}(sp)"),
        store!("a4, {a4_offset}(sp)"),
        store!("a5, {a5_offset}(sp)"),
        store!("a6, {a6_offset}(sp)"),
        store!("a7, {a7_offset}(sp)"),
        store!("s2, {s2_offset}(sp)"),
        store!("s3, {s3_offset}(sp)"),
        store!("s4, {s4_offset}(sp)"),
        store!("s5, {s5_offset}(sp)"),
        store!("s6, {s6_offset}(sp)"),
        store!("s7, {s7_offset}(sp)"),
        store!("s8, {s8_offset}(sp)"),
        store!("s9, {s9_offset}(sp)"),
        store!("s10, {s10_offset}(sp)"),
        store!("s11, {s11_offset}(sp)"),
        store!("t3, {t3_offset}(sp)"),
        store!("t4, {t4_offset}(sp)"),
        store!("t5, {t5_offset}(sp)"),
        store!("t6, {t6_offset}(sp)"),
        // With t0 freed up, we can recover the original sp value and store
        // that.
        "  addi t0, sp, {sizeof_regs}",
        store!("t0, {sp_offset}(sp)"),
        read_xepc_into_t0!(),
        store!("t0, {pc_offset}(sp)"),
        // Now we have a Regs struct that we can pass to handle_exception.
        "  mv a1, sp",
        // Before calling into Rust code we zero the frame pointerπ
        read_xcause_into_a0!(),
        // If the most-significant bit is set, then this is an interrupt. We can
        // test for this by seeing if the number is "less than zero". In that
        // case, we can also clear the top bit (to get the underlying interrupt
        // code) by shifting left and then shifting right.
        r#"
          bltz a0, .Linterrupt
          j handle_exception

        .Linterrupt:
          slli a0, a0, 1
          srli a0, a0, 1
          call handle_interrupt
        "#,
        load!("ra, {ra_offset}(sp)"),
        // We restore sp last, since everything else depends on the current
        // sp value.
        load!("gp, {gp_offset}(sp)"),
        load!("tp, {tp_offset}(sp)"),
        load!("t0, {t0_offset}(sp)"),
        load!("t1, {t1_offset}(sp)"),
        load!("t2, {t2_offset}(sp)"),
        load!("s0, {s0_offset}(sp)"),
        load!("s1, {s1_offset}(sp)"),
        load!("a0, {a0_offset}(sp)"),
        load!("a1, {a1_offset}(sp)"),
        load!("a2, {a2_offset}(sp)"),
        load!("a3, {a3_offset}(sp)"),
        load!("a4, {a4_offset}(sp)"),
        load!("a5, {a5_offset}(sp)"),
        load!("a6, {a6_offset}(sp)"),
        load!("a7, {a7_offset}(sp)"),
        load!("s2, {s2_offset}(sp)"),
        load!("s3, {s3_offset}(sp)"),
        load!("s4, {s4_offset}(sp)"),
        load!("s5, {s5_offset}(sp)"),
        load!("s6, {s6_offset}(sp)"),
        load!("s7, {s7_offset}(sp)"),
        load!("s8, {s8_offset}(sp)"),
        load!("s9, {s9_offset}(sp)"),
        load!("s10, {s10_offset}(sp)"),
        load!("s11, {s11_offset}(sp)"),
        load!("t3, {t3_offset}(sp)"),
        load!("t4, {t4_offset}(sp)"),
        load!("t5, {t5_offset}(sp)"),
        load!("t6, {t6_offset}(sp)"),
        // Restore sp last, deallocating the registers.
        load!("sp, {sp_offset}(sp)"),
        xret!(),
        sizeof_regs = const size_of::<Regs>(),
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
extern "C" fn handle_exception(code: ExceptionCode, regs: &Regs) -> ! {
    panic_common(regs.s0, Some(regs.pc), || {
        let xtval = *Xtval::read();
        print!("Exception: {code}");
        match code {
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
                if xtval == 0 {
                    print!("\n");
                } else {
                    println!(" ({xtval:#x})");
                }
            }
        }

        println!("Registers:\n{}", regs);
    });
}

#[unsafe(no_mangle)]
extern "C" fn handle_interrupt(code: InterruptCode) {
    match code {
        #[cfg(not(riscv_m_mode))]
        InterruptCode::SUPERVISOR_TIMER_INTERRUPT => {
            // TODO: no magic numbers and this should be downstream of a more
            // general policy.
            Stimecmp::from(*libarch::riscv::Time::read() + 50_000_000).write();
        }
        _ => panic!("Unexpected interrupt: {code}"),
    }
}
