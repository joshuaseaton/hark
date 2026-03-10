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

pub fn scheduler() -> &'static mut Scheduler {
    unsafe { (*&raw mut SCHEDULER).assume_init_mut() }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Created,
    Ready,
    Running,
    Exited,
}

pub struct Descriptor {
    pub context: Context,
    pub state: State,
    pub _stack: Stack,
}

pub struct Scheduler {
    pub threads: Vec<Descriptor, MAX_NUM_THREADS>,
    pub run_queue: Deque<ThreadId, MAX_NUM_THREADS>,
    pub current: ThreadId,
}

pub fn init() {
    set_scheduler(Scheduler {
        threads: Vec::from([Descriptor {
            context: Context::zero(),
            state: State::Running,
            _stack: boot_stack(),
        }]),
        run_queue: Deque::new(),
        current: ThreadId(0),
    });
}

impl Scheduler {
    pub fn get_thread(&self, id: ThreadId) -> &Descriptor {
        &self.threads[id.0]
    }

    pub fn get_thread_mut(&mut self, id: ThreadId) -> &mut Descriptor {
        &mut self.threads[id.0]
    }

    pub fn next_id(&self) -> ThreadId {
        ThreadId(self.threads.len())
    }

    pub fn switch_to(&mut self, next_id: ThreadId, old_state: State) {
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

    pub fn reschedule(&mut self, old_state: State) {
        let Some(next_id) = self.run_queue.pop_front() else {
            if old_state == State::Exited {
                panic!("all threads exited");
            }
            return;
        };
        self.switch_to(next_id, old_state);
    }
}
