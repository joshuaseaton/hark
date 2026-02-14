// Copyright (c) 2025 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

#![no_std]

/// riscv32, riscv64
pub mod riscv;

#[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))]
use riscv::Arch;

cfg_if::cfg_if! {
    if #[cfg(any(target_arch = "riscv64", target_arch = "riscv32"))] {

        // The common architectural interface. Defining the associated freeform
        // functions below in terms of a private implementation of this trait
        // allows us to document the functions in precisely one place.
        trait ArchCommon {
            fn frame_pointer() -> usize;
            fn call_frame(fp: usize) -> CallFrame;
        }

        /// The frame pointer within the scope of the caller (except in the
        /// unlikely cases where the compiler opts not to inline this
        /// `inline(always)` function, in which case the backtrace will begin
        /// in the function's own frame).
        #[inline(always)]
        pub fn frame_pointer() -> usize {
            Arch::frame_pointer()
        }

        #[derive(Clone, Copy, Debug)]
        pub(crate) struct CallFrame {
            frame_pointer: usize,
            return_address: usize,
        }

        /// An iterator yielding return addresses through a backtrace.
        pub struct Backtrace(Option<CallFrame>);

        impl Backtrace {
            /// Yields a backtrace beginning in the frame of the caller (except
            /// in unlikely cases where the compiler opts not to inline this
            /// function or [`frame_pointer`] - both marked `inline(always)` -
            /// in which case the backtrace will begin one or two frames
            /// deeper.)
            #[inline(always)]
            pub fn new() -> Self {
                Self(Some(Arch::call_frame(frame_pointer())))
            }
        }

        impl Iterator for Backtrace {
            type Item = usize;

            fn next(&mut self) -> Option<usize> {
                let CallFrame {
                    frame_pointer,
                    return_address,
                } = self.0?;
                if frame_pointer == 0 {
                    *self = Self(None);
                } else {
                    *self = Self(Some(Arch::call_frame(frame_pointer)));
                }
                Some(return_address)
            }
        }
    }
}
