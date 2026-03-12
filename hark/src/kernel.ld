/*
 * Copyright (c) 2025 Joshua Seaton
 *
 * Use of this source code is governed by a MIT-style
 * license that can be found in the LICENSE file or at
 *  https://opensource.org/licenses/MIT
 */

MEMORY {
  /* The flash range out of which the kernel is booted. */
  BOOT_FLASH (rx) : ORIGIN = BOOT_FLASH_ADDRESS, LENGTH = BOOT_FLASH_SIZE

  /* The natural range of RAM claimed by the kernel. */
  BOOT_RAM   (rw) : ORIGIN = BOOT_RAM_ADDRESS, LENGTH = BOOT_RAM_SIZE
}

ENTRY(_start)

SECTIONS {
  PROVIDE_HIDDEN(__boot_flash_start = ORIGIN(BOOT_FLASH));

  .text : {
    /* Keep _start first for use as a raw binary image */
    *(.text._start)
    *(.text*)
    *(.stext*)
  } > BOOT_FLASH

  /*
   * TODO(https://github.com/rust-lang/rust/issues/153115): Only offline unwind
   * info (in .debug_frame) should be present.
   */
  /DISCARD/ : { *(.eh_frame*) }

  .rodata : { *(.srodata*) *(.rodata*) } > BOOT_FLASH

  /* Force 4-byte alignment for the section to match the GNU Note format */
  .note.gnu.build-id : ALIGN(4) {
    __note_gnu_build_id_start = .;
    *(.note.gnu.build-id)
    __note_gnu_build_id_end = .;
  } > BOOT_FLASH
  /* The build ID note header is of a known, fixed size */
  PROVIDE_HIDDEN(__build_id_start = __note_gnu_build_id_start + 16);

  /*
   * The .data copying logic in _start relies on __data_start and __data_end
   * being 8-byte aligned.
   */
  .data : ALIGN(8) {
    PROVIDE_HIDDEN(__data_start = .);

    /*
     * Hark shell commands. Put in .data over .rodata so that the commands can
     * be sorted in the shell's initialization
     */
    __data_hark_commands_start = .;
    KEEP(*(.data.hark.commands))
    __data_hark_commands_end = .;

    . = ALIGN(8);
    *(.data*) *(.sdata*)

    . = ALIGN(8);
    PROVIDE_HIDDEN(__data_end = .);
  } > BOOT_RAM AT> BOOT_FLASH

  PROVIDE_HIDDEN(__data_lma_start = LOADADDR(.data));
  PROVIDE_HIDDEN(__boot_ram_start = ORIGIN(BOOT_RAM));
  PROVIDE_HIDDEN(__boot_flash_end = LOADADDR(.data) + SIZEOF(.data));

  /*
   * The .bss zeroing logic in _start relies on __bss_start and __bss_end
   * being 8-byte aligned.
   */
  .bss (NOLOAD) : ALIGN(8) {
    PROVIDE_HIDDEN(__bss_start = .);
    *(.bss*)
    . = ALIGN(8);
    PROVIDE_HIDDEN(__bss_end = .);
  } > BOOT_RAM

  PROVIDE_HIDDEN(__boot_ram_end = .);
}
