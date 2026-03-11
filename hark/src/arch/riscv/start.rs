// Copyright (c) 2025 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::arch::naked_asm;
use libarch::riscv::csr::Mstatus;

use crate::arch::riscv::{load, store};

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
          csrc mstatus, {mstatus_mask}
          csrw mie, zero

        // Reset the trap vector base address register, just in case a prior
        // bootloader left it set.
          csrw mtvec, zero

        // Copy .data from flash to RAM.
        // The linker script ensures that the bounds are 8-byte-aligned.
          la t0, __data_lma_start
          la t1, __data_start
          la t2, __data_end
          bge t1, t2, 1f
        0:
        "#,
        load!("t3, 0(t0)"),
        store!("t3, 0(t1)"),
        r#"
          addi t0, t0, {reg_size}
          addi t1, t1, {reg_size}
          blt t1, t2, 0b
        1:

        // Clear .bss.
        // The linker script ensures that the bounds are 8-byte-aligned.
          la t0, __bss_start
          la t1, __bss_end
          bge t0, t1, 1f
        0:
        "#,
        store!("zero, 0(t0)"),
        r#"
          add t0, t0, {reg_size}
          blt t0, t1, 0b
        1:

        // Our boot stack has been zeroed and is now ready to use.
        // Note that boot_stack_end is defined in the top-level thread module.
          la sp, boot_stack_end

        // Tail into hark_main, as there's no real benefit to keeping this
        // callframe around.
          tail hark_main
        "#,
        reg_size = const size_of::<usize>(),
        mstatus_mask = const (1 << Mstatus::MIE_BIT), // TODO: MIE_MASK
    )
}
