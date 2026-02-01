// Copyright (c) 2025 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::arch::{global_asm, naked_asm};
use cpuarch::riscv::Sstatus;

const STACK_SIZE: u64 = 0x1000; // 4KiB

// TODO: Define this via a more generic asm object macro?
global_asm!(
    r#"
    .pushsection .bss.stack, "aw", %nobits
    .balign 16
    stack:
    .skip {stack_size}
    .Lstack_end:
    .popsection
    "#,
    stack_size = const STACK_SIZE,
);

#[unsafe(no_mangle)]
#[unsafe(naked)]
extern "C" fn _start() {
    naked_asm!(
        r#"
        // Clear the return address and frame pointer: we are now at the
        // root of our call stack.
        mv ra, zero
        mv s0, zero

        // Clear any incoming stack pointer so it can't be used accidentally
        // before we set it up properly below.
        mv sp, zero

        // Clear the gp register in case anything tries to use it.
        mv gp, zero

        // Mask all interrupts in case the bootloader left them on.
        csrc sstatus, {sstatus_sie}
        csrw sie, zero

        // Reset the trap vector base address register in case the
        // bootloader left an old vector in place (which we might already be
        // clobbering, and almost certainly will be violating the
        // assumptions of).
        csrw stvec, zero

        // Disable the MMU just in case it was left on (it should not have
        // been).
        csrw satp, zero

        // Clear .bss. The linker script ensures that the start and end are
        // both 8-byte aligned.
        lla t0, __bss_start
        lla t1, __bss_end
        0:
        sd zero, (t0)
        add t0, t0, 8
        blt t0, t1, 0b

        // Our stack is now ready.
        lla sp, .Lstack_end

        call hark_main
        "#,
        sstatus_sie = const Sstatus::SIE_BIT)
}
