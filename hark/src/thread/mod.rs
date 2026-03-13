// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

mod scheduler;
use scheduler::*;

mod stack;
pub use stack::Stack;
use stack::*;

use core::ptr;

use crate::ThreadWitness;
use crate::arch::thread::Context;
use crate::heap::{self, Box};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ThreadId(usize);

/// A thread handle.
pub struct Thread {
    id: ThreadId,
}

impl Thread {
    /// Creates an unstarted thread with the given entry point and stack.
    ///
    /// The entry callable may be a capturing callable.
    ///
    /// # Panics
    ///
    /// Panics if the stack is not 16-byte aligned or is empty.
    pub fn with_stack(
        name: &'static str,
        stack: &'static mut [u8],
        entry: impl FnOnce() + Send + 'static,
    ) -> Self {
        let (fn_addr, arg_addr) = type_erase(entry);
        Self::create(name, fn_addr, arg_addr, Stack::new(stack))
    }

    /// Creates an unstarted thread with the given entry point and a
    /// dynamically allocated stack of the given size.
    ///
    /// # Panics
    ///
    /// Panics if `stack_size` is 0.
    pub fn with_stack_size(
        name: &'static str,
        stack_size: usize,
        entry: impl FnOnce() + Send + 'static,
    ) -> Self {
        assert!(stack_size > 0, "stack size must be non-zero");
        let stack = heap::allocate_stack(stack_size);
        let (fn_addr, arg_addr) = type_erase(entry);
        Self::create(name, fn_addr, arg_addr, stack)
    }

    fn create(name: &'static str, entry: usize, arg: usize, stack: Stack) -> Self {
        let ctx = Context::new(stack.top().addr(), entry, arg);
        let id = scheduler::create_thread(name, ctx, stack);
        Thread { id }
    }

    /// Begins execution of the thread.
    ///
    /// # Panics
    ///
    /// Panics if the thread has already been started.
    pub fn start(&self) {
        scheduler::start_thread(self.id);
    }
}

// Type-erases a thread entry callable into a (fn, arg) pair of raw `usize`s
// for which calling `fn` with an argument of `arg` results in calling the
// original callable. These shenanigans ensure dynamic allocation of storage for
// for any entry callable with captures.
//
// - `fn`: address of a monomorphized `call<F>` that reconstructs and
//   invokes the boxed callable.
//
// - `arg`: raw pointer to the boxed callable.
//
// For zero-sized types (i.e., function pointers and non-capturing closures),
// no allocation ensures and `arg` ends up as a dangling pointer back to the
// original callable address.
fn type_erase<F: FnOnce() + Send + 'static>(entry: F) -> (usize, usize) {
    unsafe extern "C" fn call<F: FnOnce()>(raw: *mut F) {
        let f = unsafe { ptr::read(raw) };
        f();
    }
    // Move the entry callable to the heap (unless a ZST, in which case this is
    // a no-op).
    let raw = Box::into_raw(Box::new(entry));
    (call::<F> as *const () as usize, raw as usize)
}

// Initializes threading, registering the boot context as thread 0.
pub(crate) fn init() -> ThreadWitness {
    scheduler::init();
    ThreadWitness {}
}

/// Yields the current thread, switching to the next ready thread.
///
/// No-op if no other threads are ready.
pub fn yield_now() {
    scheduler::reschedule(State::Ready);
}

pub(crate) fn thread_exit() {
    scheduler::reschedule(State::Exited);
}
