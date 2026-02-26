// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

/// Control and Status Registers
pub mod csr;

/// Supervisor Binary Interface
pub mod sbi;

#[doc(hidden)]
#[macro_export]
#[cfg(any(doc, target_arch = "riscv32", target_arch = "riscv64"))]
macro_rules! __frame_pointer {
    () => {{
        let fp: usize;
        unsafe {
            core::arch::asm!(
                "mv {}, s0",
                out(reg) fp, options(nomem, nostack, preserves_flags),
            );
        }
        fp
    }};
}

cfg_if::cfg_if! {
    if #[cfg(any(doc, target_arch = "riscv32", target_arch = "riscv64"))] {
        use core::ptr;

        use crate::{CallFrame, ArchCommon};

        pub(super) struct Arch {}

        impl ArchCommon for Arch {
            fn call_frame(fp: usize) -> CallFrame {
                unsafe {
                    let frame: *const usize = ptr::without_provenance(fp);
                    CallFrame{
                        frame_pointer: *frame.sub(2),
                        return_address: *frame.sub(1)
                    }
                }
            }
        }
    }
}
