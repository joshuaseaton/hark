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

    // Writes as many bytes to the FIFO as possible, returning true if it was
    // able to fill it fully, and thus false if the iterator was exhausted.
    //
    // Assumes the FIFO is empty.
    fn fill_fifo(io: &Self::Io, state: &Self::State, bytes: &mut impl Iterator<Item = u8>) -> bool;
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
    fn write(&self, bytes: &[u8]) {
        let mut it = CrlfByteIterator::new(bytes);
        loop {
            while !Base::tx_ready(&self.io) {}
            if !Base::fill_fifo(&self.io, &self.state, &mut it) {
                break;
            }
        }
    }
}

// A byte iterator that massages b'\n' into b"\r\n".
//
// While a standard translation done by all terminal emulators, we use this for
// compatibility in the case where there is no software layer between the UART
// and the display available to do the conversion (e.g., when hooked up to a
// physical VT100).
struct CrlfByteIterator<'a> {
    bytes: &'a [u8],
    pos: usize,

    // Whether the last byte seen was b'\r'.
    prev_cr: bool,
}

impl<'a> CrlfByteIterator<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            pos: 0,
            prev_cr: false,
        }
    }
}

impl Iterator for CrlfByteIterator<'_> {
    type Item = u8;

    fn next(&mut self) -> Option<u8> {
        if self.pos >= self.bytes.len() {
            return None;
        }
        let next = self.bytes[self.pos];
        if next == b'\n' {
            if !self.prev_cr {
                self.prev_cr = true;
                return Some(b'\r');
            }
            self.prev_cr = false;
        }
        self.prev_cr = next == b'\r';
        self.pos += 1;
        Some(next)
    }
}
