/*
 * Copyright (c) 2025 Joshua Seaton
 *
 * Use of this source code is governed by a MIT-style
 * license that can be found in the LICENSE file or at
 *  https://opensource.org/licenses/MIT
 */

ENTRY(_start)

SECTIONS {
  . = LOAD_ADDRESS;
  PROVIDE_HIDDEN(__executable_start = .);


  .text : {
    /* Keep _start first for use as a raw binary image */
    *(.text._start)
    *(.text*)
    *(.stext*)
  }

  /*
   * Only offline unwind info (in .debug_frame) should be present.
   *
   * TODO: link to the Rust issue preventing this section from being empty when
   * linking against the alloc crate, and go back to asserting emptiness once
   * resolved.
   */
  /DISCARD/ : { *(.eh_frame*) }

  .rodata : { *(.srodata*) *(.rodata*) }

  /* Force 4-byte alignment for the section to match the GNU Note format */
  .note.gnu.build-id : ALIGN(4) {
    __note_gnu_build_id_start = .;
    *(.note.gnu.build-id)
    __note_gnu_build_id_end = .;
  }


  .data : { *(.data*) *(.sdata*) }

   /*
    * The .bss zeroing logic in start.S relies on __bss_start and __bss_end
    * being 8-byte aligned.
    */
  .bss : ALIGN(8) {
    PROVIDE_HIDDEN(__bss_start = .);
    *(.bss*)
    . = ALIGN(8);
    PROVIDE_HIDDEN(__bss_end = .);
  }
  
  PROVIDE_HIDDEN(_end = .);
}
