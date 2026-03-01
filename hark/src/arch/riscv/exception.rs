// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::arch::naked_asm;
use core::mem::offset_of;

use libarch::riscv::csr::{ExceptionCode, InterruptCode, TrapVectorMode};
use regio::Register as _;

use crate::arch::riscv::{
    Regs, Xcause, Xepc, Xie, Xstatus, Xtvec, disable_interrupts, enable_interrupts,
};
use crate::kernel::panic_common;
use crate::platform;
use crate::{print, println};

cfg_if::cfg_if! {
    if #[cfg(target_pointer_width = "32")] {
        macro_rules! store { ($args:literal) => { concat!("sw ", $args) }; }
        macro_rules! load { ($args:literal) => { concat!("lw ", $args) }; }
    } else {
        macro_rules! store { ($args:literal) => { concat!("sd ", $args) }; }
        macro_rules! load { ($args:literal) => { concat!("ld ", $args) }; }
    }
}

cfg_if::cfg_if! {
    if #[cfg(riscv_m_mode)] {
        macro_rules! read_xepc_into_t0 { () => { "csrr t0, mepc" }; }
        macro_rules! read_xcause_into_t0 { () => { "csrr t0, mcause" }; }
        macro_rules! read_xstatus_into_t0 { () => { "csrr t0, mstatus" }; }
        macro_rules! read_xtval_into_t0 { () => { "csrr t0, mtval" }; }

        macro_rules! xret { () => { "mret" }; }

        fn restore_xstatus_bits(old: Xstatus) {
            Xstatus::modify(|reg| {
                reg.set_mpie(old.mpie()).set_mpp(old.mpp());
            });
        }

        const XSTATUS: &str = "sstatus";
    } else {
        use libarch::riscv::csr::Stimecmp;

        macro_rules! read_xepc_into_t0 { () => { "csrr t0, sepc" }; }
        macro_rules! read_xcause_into_t0 { () => { "csrr t0, scause" }; }
        macro_rules! read_xstatus_into_t0 { () => { "csrr t0, sstatus" }; }
        macro_rules! read_xtval_into_t0 { () => { "csrr t0, stval" }; }

        macro_rules! xret { () => { "sret" }; }

        fn restore_xstatus_bits(old: Xstatus) {
            Xstatus::modify(|reg| {
                reg.set_spie(old.spie()).set_spp(old.spp());
            });
        }

        const XSTATUS: &str = "mstatus";
    }
}

pub(super) fn init() {
    let entry = (exception_entry as *const ()).addr();
    Xtvec::from(0)
        .set_base(entry)
        .set_mode(TrapVectorMode::Direct)
        .write();

    // Enable all supported interrupts.
    // TODO: support more!
    cfg_if::cfg_if! {
        if #[cfg(riscv_m_mode)] {
            Xie::from(0).set_meie(true).write();
        } else {
            Xie::from(0).set_stie(true).set_seie(true).write();
        }
    }
    enable_interrupts();
}

#[repr(C)]
struct TrapFrame {
    regs: Regs,
    xstatus: Xstatus,
    xcause: Xcause,
    xtval: usize,
    _padding: usize,
}

// We set sp to the beginning of an allocated TrapFrame below and sp is required
// to be 16-byte aligned at call boundaries.
const _: () = const { assert!(size_of::<TrapFrame>().is_multiple_of(16)) };

#[unsafe(no_mangle)]
#[unsafe(naked)]
extern "C" fn exception_entry() {
    naked_asm!(
        // We save registers onto the stack, making sure not to trash any before
        // we do.
        //
        // Save pc for later, when we have a spare register for reading in xepc.
        "  addi sp, sp, -{sizeof_frame}",
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
        "  addi t0, sp, {sizeof_frame}",
        store!("t0, {sp_offset}(sp)"),
        read_xepc_into_t0!(),
        store!("t0, {pc_offset}(sp)"),
        read_xstatus_into_t0!(),
        store!("t0, {xstatus_offset}(sp)"),
        read_xtval_into_t0!(),
        store!("t0, {xtval_offset}(sp)"),
        read_xcause_into_t0!(), // Intentionally last.
        store!("t0, {xcause_offset}(sp)"),

        // Now we have a TrapFrame that we can pass to handle_exception() and
        // handle_interrupt().
        "  mv a0, sp",
        // Before calling into Rust code we zero the frame pointer.
        "  mv fp, x0",
        // Recall that t0 still holds xcause. If the most-significant bit is
        // set, then this is an interrupt. We can test for this by seeing if the
        // number is "less than zero".
        r#"
          bltz t0, .Linterrupt
          j handle_exception

        .Linterrupt:
          call handle_interrupt
        "#,

        // On handle_interrupt() exit, interrupts are disabled and CSR state has
        // been restored, leaving the registers.

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
        // Restore sp last, deallocating the frame.
        load!("sp, {sp_offset}(sp)"),
        xret!(),
        sizeof_frame = const size_of::<TrapFrame>(),
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
        xstatus_offset = const offset_of!(TrapFrame, xstatus),
        xcause_offset = const offset_of!(TrapFrame, xcause),
        xtval_offset = const offset_of!(TrapFrame, xtval),
    )
}

#[unsafe(no_mangle)]
extern "C" fn handle_exception(frame: &TrapFrame) -> ! {
    panic_common(frame.regs.s0, Some(frame.regs.pc), || {
        let code = frame.xcause.exception_code();
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
                println!(" (@ {:#x})", frame.xtval);
            }
            _ => {
                if frame.xtval == 0 {
                    print!("\n");
                } else {
                    println!(" ({:#x})", frame.xtval);
                }
            }
        }

        println!("Registers:\n{}", frame.regs);
    });
}

// On exit of this function, interrupts will be disabled and the CSR state
// relevant to xret (i.e., xepc, and the xpie and xpp bits of xstatus) will have
// been preserved.
#[unsafe(no_mangle)]
extern "C" fn handle_interrupt(frame: &TrapFrame) {
    let code = frame.xcause.interrupt_code();
    match code {
        #[cfg(not(riscv_m_mode))]
        InterruptCode::SUPERVISOR_TIMER_INTERRUPT => {
            // TODO: no magic numbers and this should be downstream of a more
            // general policy.
            Stimecmp::from(*libarch::riscv::csr::Time::read() + 50_000_000).write();
        }
        #[cfg(not(riscv_m_mode))]
        InterruptCode::SUPERVISOR_EXTERNAL_INTERRUPT => {
            handle_external_interrupt(frame);
        }
        #[cfg(riscv_m_mode)]
        InterruptCode::MACHINE_EXTERNAL_INTERRUPT => {
            handle_external_interrupt(frame);
        }
        _ => panic_common(frame.regs.s0, Some(frame.regs.pc), || {
            println!("Unsupported interrupt: {code}");
            println!("{XSTATUS}: {:#x}", *frame.xstatus);
            println!("Registers:\n{}", frame.regs);
        }),
    }
}

fn handle_external_interrupt(frame: &TrapFrame) {
    let irq = platform::interrupt::claim_pending_irq();

    // External interrupts are fully pre-emptible in Hark.
    enable_interrupts();
    platform::interrupt::handle(irq);
    disable_interrupts();

    // Now that interrupts are disabled, we restore xepc and the
    // xstatus.{xpie, xpp} ahead of the xret. Easier to do that here then in the
    // assembly epilogue.
    Xepc::from(frame.regs.pc).write();
    restore_xstatus_bits(frame.xstatus);
}
