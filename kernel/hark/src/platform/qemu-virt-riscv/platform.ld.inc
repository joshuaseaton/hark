/*
 * Copyright (c) 2025 Joshua Seaton
 *
 * Use of this source code is governed by a MIT-style
 * license that can be found in the LICENSE file or at
 *  https://opensource.org/licenses/MIT
 */

#define MEMBASE 0x80000000

#ifdef CARGO_FEATURE_RISCV_M_MODE
#define KERNEL_LOAD_OFFSET 0
#else
/* Accounts for OpenSBI being loaded at the base of RAM. */
#define KERNEL_LOAD_OFFSET 0x200000
#endif
