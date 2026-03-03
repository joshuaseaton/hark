// Copyright (c) 2025 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::arch::{global_asm, naked_asm};
use libarch::riscv::csr::Mstatus;

const STACK_SIZE: u64 = 0x2000; // 8KiB

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
        "#,
        r#"
        // Mask all interrupts in case a prior bootloader left them.
        csrc mstatus, {mstatus_mie}
        csrw mie, zero

        // Reset the trap vector base address register, just in case a prior
        // bootloader left it set.
        csrw mtvec, zero

        // Clear .bss. The linker script ensures that the start and end are
        // both 8-byte aligned.
        la t0, __bss_start
        la t1, __bss_end
        0:
        sw zero, 0(t0)
        add t0, t0, {word_size}
        blt t0, t1, 0b

        // Our stack is now ready.
        la sp, .Lstack_end

        // Tail into hark_main, as there's real benefit to keeping this
        // callframe around.
        call hark_main
        "#,
        word_size = const size_of::<usize>(),
        mstatus_mie = const Mstatus::MIE_BIT,
    )
}
