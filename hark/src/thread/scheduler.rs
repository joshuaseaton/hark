// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::mem::MaybeUninit;

use heapless::{Deque, Vec};

use super::{Stack, ThreadId, boot_stack};
use crate::arch::thread::Context;
use crate::println;
use crate::shell::{self, Args};
use crate::sync::InterruptGuard;

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

impl State {
    const fn as_str(self) -> &'static str {
        match self {
            State::Created => "created",
            State::Ready => "ready",
            State::Running => "running",
            State::Exited => "exited",
        }
    }
}

struct Descriptor {
    pub name: &'static str,
    pub context: Context,
    pub state: State,
    pub stack: Stack,
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
            name: "boot",
            context: Context::zero(),
            state: State::Running,
            stack: boot_stack(),
        }]),
        run_queue,
        current,
    });
}

pub fn create_thread(name: &'static str, context: Context, stack: Stack) -> ThreadId {
    scheduler().create_thread(name, context, stack)
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

    fn create_thread(&mut self, name: &'static str, context: Context, stack: Stack) -> ThreadId {
        let desc = Descriptor {
            name,
            context,
            state: State::Created,
            stack,
        };
        {
            let _guard = InterruptGuard::new();

            let id = ThreadId(self.threads.len());
            let _ = self.threads.push(desc);
            id
        }
    }

    fn start_thread(&mut self, id: ThreadId) {
        let _guard = InterruptGuard::new();

        let desc = self.get_thread_mut(id);
        assert_eq!(desc.state, State::Created, "thread already started");
        desc.state = State::Ready;
        let _ = self.run_queue.push_back(id);
    }

    fn switch_to(&mut self, _: &InterruptGuard, next_id: ThreadId, old_state: State) {
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
        let guard = InterruptGuard::new();

        let Some(next_id) = self.run_queue.pop_front() else {
            assert!(old_state != State::Exited, "all threads exited");
            return;
        };
        if next_id == self.current {
            return;
        }
        self.switch_to(&guard, next_id, old_state);
    }
}

/// thread {list}
///
/// `list` will list all threads in the system.
#[shell::command(help = "Inspect the threads in the system")]
fn thread(mut args: Args) -> bool {
    let Some(subcommand) = args.next() else {
        return false;
    };
    if subcommand != "list" || args.next().is_some() {
        return false;
    }
    let _guard = InterruptGuard::new();
    for desc in &scheduler().threads {
        println!(
            " * {}: {}, stack size = {:#x}",
            desc.name,
            desc.state.as_str(),
            desc.stack.size()
        );
    }
    true
}
