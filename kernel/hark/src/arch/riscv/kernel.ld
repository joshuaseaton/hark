/*
 * Copyright (c) 2025 Joshua Seaton
 *
 * Use of this source code is governed by a MIT-style
 * license that can be found in the LICENSE file or at
 *  https://opensource.org/licenses/MIT
 */

/* */
#include "platform.ld.inc"

ENTRY(_start)

SECTIONS {
  . = MEMBASE + KERNEL_LOAD_OFFSET;

  .text : {
    /* Keep _start first for use as a raw binary image */
    *(.text._start)
    *(.text*)
    *(.stext*)
  }

  /* Only offline unwind info (in .debug_frame) should be present. */
  .eh_frame : { *(.eh_frame*) }
  ASSERT(SIZEOF(.eh_frame) == 0, "no .eh_frame should be emitted")

  .rodata : { *(.srodata*) *(.rodata*) }

  .data : { *(.data*) *(.sdata*) }

  /* TODO: What alignment guarantees do I have with the start of a section? */
  /* TODO: Phys code in Fuchsia aligns to 16 bytes - why? */
  .bss : {
    PROVIDE_HIDDEN(__bss_start = .);
    *(.bss*)
    /*
     * The .bss zeroing logic in start.S relies on __bss_start and __bss_end
     * being 8-byte aligned.
     */
    . = ALIGN(8);
    PROVIDE_HIDDEN(__bss_end = .);
  }
}
