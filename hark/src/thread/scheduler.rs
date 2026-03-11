// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::mem::MaybeUninit;

use heapless::{Deque, Vec};

use super::{Stack, ThreadId, boot_stack};
use crate::arch::thread::Context;

// TODO: parameterize via environment variable.
const MAX_NUM_THREADS: usize = 32;

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
pub enum State {
    Created,
    Ready,
    Running,
    Exited,
}

struct Descriptor {
    pub context: Context,
    pub state: State,
    pub _stack: Stack,
}

struct Scheduler {
    pub threads: Vec<Descriptor, MAX_NUM_THREADS>,
    pub run_queue: Deque<ThreadId, MAX_NUM_THREADS>,
    pub current: ThreadId,
}

pub fn init() {
    let current = ThreadId(0);
    let mut run_queue = Deque::new();
    run_queue.push_back(current).unwrap();
    set_scheduler(Scheduler {
        threads: Vec::from([Descriptor {
            context: Context::zero(),
            state: State::Running,
            _stack: boot_stack(),
        }]),
        run_queue,
        current,
    });
}

pub fn create_thread(context: Context, stack: Stack) -> ThreadId {
    scheduler().create_thread(context, stack)
}

pub fn start_thread(id: ThreadId) {
    scheduler().start_thread(id);
}

pub fn reschedule(old_state: State) {
    scheduler().reschedule(old_state);
}

impl Scheduler {
    fn get_thread(&self, id: ThreadId) -> &Descriptor {
        &self.threads[id.0]
    }

    fn get_thread_mut(&mut self, id: ThreadId) -> &mut Descriptor {
        &mut self.threads[id.0]
    }

    fn create_thread(&mut self, context: Context, stack: Stack) -> ThreadId {
        let desc = Descriptor {
            context,
            state: State::Created,
            _stack: stack,
        };
        // TODO: critical section start.
        let id = ThreadId(self.threads.len());
        let _ = self.threads.push(desc);
        // TODO: critical section end.
        id
    }

    fn start_thread(&mut self, id: ThreadId) {
        // TODO: critical section start.
        let desc = self.get_thread_mut(id);
        assert_eq!(desc.state, State::Created, "thread already started");
        desc.state = State::Ready;
        let _ = self.run_queue.push_back(id);
        // TODO: critical section end.
    }

    fn switch_to(&mut self, next_id: ThreadId, old_state: State) {
        // TODO: critical section
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

    fn reschedule(&mut self, old_state: State) {
        // TODO: critical section
        let Some(next_id) = self.run_queue.pop_front() else {
            assert!(old_state != State::Exited, "all threads exited");
            return;
        };
        if next_id == self.current {
            return;
        }
        self.switch_to(next_id, old_state);
    }
}
