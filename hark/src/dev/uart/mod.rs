// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

pub mod ns8250;

use core::fmt;

use regio::{IoBackend, Mmio};

use crate::dev::Console;

pub type Ns8250 = Driver<ns8250::Base<Mmio<u8>>>;

pub trait DriverBase {
    type Io: IoBackend;
    type State: Default;

    fn io(base: usize) -> Self::Io;
    fn describe(w: &mut impl fmt::Write, state: &Self::State);
    fn init(io: &Self::Io, state: &mut Self::State);
    fn tx_ready(io: &Self::Io) -> bool;

    // Assumes the FIFO is empty at the time of write.
    fn write<'a>(io: &Self::Io, state: &Self::State, bytes: &'a [u8]) -> &'a [u8];
}

// A generic UART driver.
pub struct Driver<Base: DriverBase> {
    io: Base::Io,
    state: Base::State,
}

impl<Base: DriverBase> Driver<Base> {
    // Creates an initialized UART object over an MMIO aperture.
    pub fn new(base: usize) -> Self {
        let io = Base::io(base);
        let mut state = Base::State::default();
        Base::init(&io, &mut state);
        Self { io, state }
    }
}

impl<Base: DriverBase> Console for Driver<Base> {
    fn describe(&self, w: &mut impl fmt::Write) {
        Base::describe(w, &self.state);
    }

    // Writes the provided bytes to the UART.
    #[inline]
    fn write(&self, mut bytes: &[u8]) {
        while !bytes.is_empty() {
            while !Base::tx_ready(&self.io) {}
            bytes = Base::write(&self.io, &self.state, bytes);
        }
    }
}
