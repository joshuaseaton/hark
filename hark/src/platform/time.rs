// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use super::backend;
use crate::arch;

// TODO: integer division nuance.

/// System time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Time {
    ticks: u64,
}

impl Time {
    const MILLISECONDS_PER_SECOND: u64 = 1_000;
    const MICROSECONDS_PER_SECOND: u64 = 1_000_000;

    /// The time in seconds.
    pub const fn from_seconds(secs: u64) -> Self {
        Self {
            ticks: secs * backend::TIMER_FREQUENCY,
        }
    }

    /// The time in milliseconds.
    pub const fn from_milliseconds(msecs: u64) -> Self {
        Self {
            ticks: (msecs * backend::TIMER_FREQUENCY) / Self::MILLISECONDS_PER_SECOND,
        }
    }

    /// The time in microseconds.
    pub const fn from_microseconds(usecs: u64) -> Self {
        Self {
            ticks: (usecs * backend::TIMER_FREQUENCY) / Self::MICROSECONDS_PER_SECOND,
        }
    }

    /// Current system time.
    pub fn now() -> Self {
        Self {
            ticks: arch::get_ticks(),
        }
    }

    /// Convert to system ticks.
    pub const fn to_ticks(self) -> u64 {
        self.ticks
    }
}
