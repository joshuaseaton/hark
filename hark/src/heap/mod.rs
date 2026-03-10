// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::mem::{self, MaybeUninit};
use core::ops::{Deref, DerefMut};
use core::ptr::{self, NonNull};
use core::slice;

use crate::platform;

unsafe extern "C" {
    static __boot_ram_end: u8;
}

static mut ALLOCATOR: MaybeUninit<BumpAllocator> = MaybeUninit::uninit();

fn allocator() -> &'static mut BumpAllocator {
    unsafe { (*&raw mut ALLOCATOR).assume_init_mut() }
}

#[derive(Clone, Copy)]
pub(crate) struct Range {
    pub start: usize,
    pub size: usize,
}

impl Range {
    pub const fn end(self) -> usize {
        self.start + self.size
    }
}

/// A represents an allocated stack.
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

/// An owning pointer to a heap-allocated value of type `T`.
///
/// Similar to `alloc::boxed::Box`, but backed by the kernel's heap. The
/// The destructor for `T` runs on drop, but the memory is not reclaimed.
pub struct Box<T> {
    ptr: NonNull<T>,
}

impl<T> Box<T> {
    /// Moves the provided value into allocated memory and returns the owning
    /// [`Box`].
    ///
    /// Zero-sized types are accepted and do not result in heap allocation.
    pub fn new(value: T) -> Self {
        let ptr = if size_of::<T>() == 0 {
            NonNull::dangling()
        } else {
            let mem = allocate(size_of::<T>(), align_of::<T>());
            unsafe { NonNull::new_unchecked(mem.as_mut_ptr().cast::<T>()) }
        };
        unsafe { ptr.as_ptr().write(value) };
        Self { ptr }
    }

    /// Constructs a [`Box`] from a raw pointer.
    ///
    /// # Safety
    ///
    /// `ptr` must have been obtained from [`Box::into_raw`].
    ///
    /// # Panics
    ///
    /// Panics if `ptr` is null.
    pub unsafe fn from_raw(ptr: *mut T) -> Self {
        assert!(!ptr.is_null());
        unsafe {
            Self {
                ptr: NonNull::new_unchecked(ptr),
            }
        }
    }

    /// Consumes the [`Box`], returning a raw pointer to the allocation.
    ///
    /// The caller is responsible for the memory and the value; the
    /// destructor for `T` will not be run.
    pub fn into_raw(mut self) -> *mut T {
        unsafe { self.ptr.as_mut() }
    }

    /// Consumes the [`Box`], returning the contained value.
    pub fn into_inner(self) -> T {
        let val = unsafe { self.ptr.as_ptr().read() };
        mem::forget(self);
        val
    }
}

impl<T> Deref for Box<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { self.ptr.as_ref() }
    }
}

impl<T> DerefMut for Box<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut() }
    }
}

impl<T> Drop for Box<T> {
    fn drop(&mut self) {
        unsafe { ptr::drop_in_place(self.ptr.as_ptr()) }
    }
}

struct BumpAllocator {
    front: usize,
    back: usize,
}

// Initializes the heap, claiming all RAM outside of the kernel's static
// allocations.
pub(crate) fn init() {
    let ram = platform::RAM;
    let front = (&raw const __boot_ram_end).addr();
    let back = ram.end();
    debug_assert!(front >= ram.start && front <= back);
    unsafe {
        (*&raw mut ALLOCATOR).write(BumpAllocator { front, back });
    }
}

/// Allocates memory of a given size and alignment in bytes.
///
/// # Panics
///
/// Panics on OOM.
pub fn allocate(size: usize, align: usize) -> &'static mut [u8] {
    assert!(size > 0);
    assert!(align.is_power_of_two());

    let alloc = allocator();
    let start = alloc.front.next_multiple_of(align);
    let end = start + size;
    assert!(end <= alloc.back, "OOM!");

    alloc.front = end;
    unsafe { slice::from_raw_parts_mut(start as *mut u8, size) }
}
/// Allocates a stack of the given size in bytes (necessarily 16-byte aligned).
///
/// # Panics
///
/// Panics on OOM.
pub fn allocate_stack(size: usize) -> Stack {
    Stack::new(allocate(size, 16))
}
