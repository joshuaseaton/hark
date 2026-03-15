// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::fmt;

use regio::Mmio;

use super::DriverBase;

/// Used only during bring-up as a convenient placeholder.
pub struct Base {}

impl DriverBase for Base {
    type Io = Mmio<u32>;

    type State = ();

    fn io(_: usize) -> Self::Io {
        Mmio::new(0, 0)
    }

    fn describe(_: &mut impl fmt::Write, (): &Self::State) {}

    fn init(_: &Self::Io, (): &mut Self::State) {}

    fn tx_fifo_is_empty(_: &Self::Io) -> bool {
        true
    }
    fn read_byte(_: &Self::Io) -> Option<u8> {
        None
    }
    fn fill_empty_tx_fifo(
        _: &Self::Io,
        (): &Self::State,
        bytes: &mut impl Iterator<Item = u8>,
    ) -> bool {
        for _ in bytes {}
        false
    }
}
