// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

/// Control and Status Registers
pub mod csr;

mod sbi;
pub use sbi::*;

cfg_if::cfg_if! {
    if #[cfg(any(doc, target_arch = "riscv32", target_arch = "riscv64"))] {
        #[doc(inline)]
        pub use crate::sbi_call;

        use core::{arch::asm, ptr};

        use crate::{CallFrame, ArchCommon};

        pub(super) struct Arch {}

        impl ArchCommon for Arch {

            #[inline(always)]
            fn frame_pointer() -> usize {
                let mut fp: usize;
                unsafe {
                    asm!("mv {}, s0", out(reg) fp);
                }
                fp
            }

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
