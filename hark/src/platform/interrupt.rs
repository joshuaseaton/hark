// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::mem::MaybeUninit;

use crate::platform::backend;

static mut DISPATCHER: MaybeUninit<Dispatcher> = MaybeUninit::uninit();

fn set_dispatcher(dispatcher: Dispatcher) {
    unsafe {
        (*&raw mut DISPATCHER).write(dispatcher);
    }
}

fn get_dispatcher() -> &'static Dispatcher {
    unsafe { (*&raw const DISPATCHER).assume_init_ref() }
}

struct Dispatcher {
    controller: backend::InterruptController,

    // Handlers indexed by IRQ number.
    handlers: [fn(); backend::MAX_IRQ as usize],
}

pub fn init() {
    set_dispatcher(Dispatcher {
        controller: backend::interrupt_controller(),
        handlers: [|| unimplemented!(); backend::MAX_IRQ as usize],
    });
}

pub fn claim_pending_irq() -> u32 {
    get_dispatcher()
        .controller
        .claim_pending_irq()
        .expect("no pending interrupt!")
}

pub fn handle(irq: u32) {
    let dispatcher = get_dispatcher();
    dispatcher.handlers[irq as usize]();
    dispatcher.controller.complete_irq(irq);
}
