// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::arch::global_asm;
use core::slice;

// TODO: parameterize via environment variable.
pub const BOOT_STACK_SIZE: u64 = 0x1800; // 6KiB

// The boot stack.
global_asm!(
    r#"
    .pushsection .bss.stack, "aw", %nobits
    .balign 16
    .global boot_stack_start
    boot_stack_start:
    .skip {stack_size}
    .global boot_stack_end
    boot_stack_end:
    .popsection
    "#,
    stack_size = const BOOT_STACK_SIZE,
);

unsafe extern "C" {
    static boot_stack_start: u8;
}

pub fn boot_stack() -> Stack {
    let stack = unsafe {
        slice::from_raw_parts_mut(
            (&raw const boot_stack_start).cast_mut(),
            BOOT_STACK_SIZE as usize,
        )
    };
    Stack::new(stack)
}

/// Represents an allocated stack.
#[derive(Clone, Copy, Debug)]
pub struct Stack {
    base: *mut u8,
    size: usize,
}

impl Stack {
    pub(crate) fn new(stack: &'static mut [u8]) -> Self {
        let base = stack.as_mut_ptr();
        let top = base.addr() + stack.len();
        assert!(top.is_multiple_of(16), "stack must be 16-byte aligned");
        assert!(!stack.is_empty(), "stack must be non-empty");
        Self {
            base,
            size: stack.len(),
        }
    }

    /// The base of the stack.
    pub const fn base(&self) -> *mut u8 {
        self.base
    }

    /// The top of the stack.
    pub const fn top(&self) -> *mut u8 {
        unsafe { self.base.add(self.size) }
    }

    /// The size of the stack.
    pub const fn size(&self) -> usize {
        self.size
    }
}
