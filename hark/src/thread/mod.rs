// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::arch::global_asm;
use core::mem::MaybeUninit;
use core::{ptr, slice};

use heapless::{Deque, Vec};

use crate::arch::thread::Context;
use crate::heap::{self, Box, Stack};

// TODO: parameterize via environment variable.
const MAX_NUM_THREADS: usize = 32;

// TODO: parameterize via environment variable.
pub(crate) const BOOT_STACK_SIZE: u64 = 0x1000;

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

static mut SCHEDULER: MaybeUninit<Scheduler> = MaybeUninit::uninit();

fn set_scheduler(sched: Scheduler) {
    unsafe {
        (*&raw mut SCHEDULER).write(sched);
    }
}

fn scheduler() -> &'static mut Scheduler {
    unsafe { (*&raw mut SCHEDULER).assume_init_mut() }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ThreadId(usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    Created,
    Ready,
    Running,
    Exited,
}

struct Descriptor {
    context: Context,
    state: State,
    _stack: Stack,
}

struct Scheduler {
    threads: Vec<Descriptor, MAX_NUM_THREADS>,
    run_queue: Deque<ThreadId, MAX_NUM_THREADS>,
    current: ThreadId,
}

impl Scheduler {
    fn get_thread(&self, id: ThreadId) -> &Descriptor {
        &self.threads[id.0]
    }

    fn get_thread_mut(&mut self, id: ThreadId) -> &mut Descriptor {
        &mut self.threads[id.0]
    }

    fn switch_to(&mut self, next_id: ThreadId, old_state: State) {
        let current_id = self.current;
        self.get_thread_mut(current_id).state = old_state;
        if old_state == State::Ready {
            let _ = self.run_queue.push_back(current_id);
        }
        self.current = next_id;
        self.get_thread_mut(next_id).state = State::Running;

        let old = &raw mut self.get_thread_mut(current_id).context;
        let new = &raw const self.get_thread(next_id).context;
        unsafe { (*old).switch(&*new) };
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
    pub fn with_stack(stack: &'static mut [u8], entry: impl FnOnce() + Send + 'static) -> Self {
        let (fn_addr, arg_addr) = type_erase(entry);
        Self::create(fn_addr, arg_addr, Stack::new(stack))
    }

    /// Creates an unstarted thread with the given entry point and a
    /// dynamically allocated stack of the given size.
    ///
    /// # Panics
    ///
    /// Panics if `stack_size` is 0.
    pub fn with_stack_size(stack_size: usize, entry: impl FnOnce() + Send + 'static) -> Self {
        assert!(stack_size > 0, "stack size must be non-zero");
        let stack = heap::allocate_stack(stack_size);
        let (fn_addr, arg_addr) = type_erase(entry);
        Self::create(fn_addr, arg_addr, stack)
    }

    fn create(entry: usize, arg: usize, stack: Stack) -> Self {
        let ctx = Context::new(stack.top().addr(), entry, arg);
        let sched = scheduler();
        let id = ThreadId(sched.threads.len());
        let _ = sched.threads.push(Descriptor {
            context: ctx,
            state: State::Created,
            _stack: stack,
        });
        Thread { id }
    }

    /// Begins execution of the thread.
    ///
    /// # Panics
    ///
    /// Panics if the thread has already been started.
    pub fn start(&self) {
        let sched = scheduler();
        let desc = sched.get_thread_mut(self.id);
        assert_eq!(desc.state, State::Created, "thread already started");
        desc.state = State::Ready;
        let _ = sched.run_queue.push_back(self.id);
    }
}

/// Yields the current thread, switching to the next ready thread.
///
/// No-op if no other threads are ready.
pub fn yield_now() {
    let sched = scheduler();
    let Some(next_id) = sched.run_queue.pop_front() else {
        return;
    };
    sched.switch_to(next_id, State::Ready);
}

// Initializes threading, registering the boot context as thread 0.
pub(crate) fn init() {
    let boot_stack = unsafe {
        slice::from_raw_parts_mut(
            (&raw const boot_stack_start).cast_mut(),
            BOOT_STACK_SIZE as usize,
        )
    };
    set_scheduler(Scheduler {
        threads: Vec::from([Descriptor {
            context: Context::zero(),
            state: State::Running,
            _stack: Stack::new(boot_stack),
        }]),
        run_queue: Deque::new(),
        current: ThreadId(0),
    });
}

pub(crate) fn thread_exit() {
    let sched = scheduler();
    let Some(next_id) = sched.run_queue.pop_front() else {
        panic!("all threads exited");
    };
    sched.switch_to(next_id, State::Exited);
}
