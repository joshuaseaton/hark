// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::mem::MaybeUninit;

use derive_more::{Deref, From};
use regio::{Mmio, Register as _, array, offset};

use crate::arch;
use crate::platform::{RISCV_MTIMER_TIME_ADDRESS, RISCV_MTIMER_TIMECMP_ADDRESS};

const TIMECMP_APERTURE_SIZE: usize = 0x7ff8;

static mut TIMER: MaybeUninit<Mtimer> = MaybeUninit::uninit();

fn set_timer(timer: Mtimer) {
    unsafe {
        (*&raw mut TIMER).write(timer);
    }
}

fn get_timer() -> &'static Mtimer {
    unsafe { (*&raw const TIMER).assume_init_ref() }
}

pub fn init() {
    set_timer(Mtimer::new(
        RISCV_MTIMER_TIME_ADDRESS,
        RISCV_MTIMER_TIMECMP_ADDRESS,
    ));
}

pub fn handle_exception() {}

#[offset(0)]
#[repr(transparent)]
#[derive(Debug, Deref, From)]
struct Mtime(u64);

#[array(0x8)]
#[repr(transparent)]
#[derive(Debug, Deref, From)]
struct Mtimecmp(u64);

cfg_if::cfg_if! {
    if #[cfg(target_arch = "riscv32")] {
        use regio::{IoBackend, Offset};

        struct MtimeIo(Mmio<u32>);

        impl MtimeIo {
            fn new(base: usize, size: usize) -> Self {
                debug_assert_eq!(size, size_of::<Mtime>());
                Self(Mmio::new(base, size))
            }
        }

        impl IoBackend for MtimeIo {
            type Addr = Offset;
            type Base = u64;

            #[inline]
            fn read_at(&self, addr: Offset) -> u64 {
                let low_addr = addr;
                let high_addr = Offset(*addr + size_of::<u32>());

                // Unlikely, but mtime might have wrapped around in between
                // reading high and reading low. We can detect this by reading
                // high again and comparing.
                let mut high = self.0.read_at(high_addr);
                let mut low = self.0.read_at(low_addr);
                let high2 = self.0.read_at(high_addr);
                if high2 != high { // TODO: core::hint::unlikely() when stable
                    // Wrapped around! Read out low again as well.
                    high = high2;
                    low = self.0.read_at(low_addr);
                }
                (u64::from(high) << 32) | u64::from(low)
            }

            #[inline]
            fn write_at(&self, value: u64, addr: Offset) {
                let low_addr = addr;
                let high_addr = Offset(*addr + size_of::<u32>());

                // In order to not have the timer spuriously fire while we
                // update mtime, we need to minimize the value it holds as we
                // update it.

                // Accordingly, zero out the upper half and update it last.
                self.0.write_at(0, high_addr);

                let low = value as u32;
                self.0.write_at(low, low_addr);

                // At this point the timer could have fired if low >= mtimecmp,
                // but in that case it would have fired anyway if high had also
                // been in place. It would also be strange to have high != 0
                // with mtimecmp so small.

                let high = (value >> 32) as u32;
                self.0.write_at(high, high_addr);
            }
        }

        struct MtimecmpIo(Mmio<u32>);

        impl MtimecmpIo {
            fn new(base: usize, size: usize) -> Self {
                debug_assert_eq!(size, TIMECMP_APERTURE_SIZE);
                Self(Mmio::new(base, size))
            }
        }

        impl IoBackend for MtimecmpIo {
            type Addr = Offset;
            type Base = u64;

            #[inline]
            fn read_at(&self, addr: Offset) -> u64 {
                let low = self.0.read_at(addr);
                let high = self.0.read_at(Offset(*addr + size_of::<u32>()));
                (u64::from(high) << 32) | u64::from(low)
            }

            #[inline]
            fn write_at(&self, value: u64, addr: Offset) {
                let low_addr = addr;
                let high_addr = Offset(*addr + size_of::<u32>());

                // Similar to the above, in order to not have the timer
                // spuriously fire while we update mtimecmp, we need to maximize
                // the value it holds as we update it.

                // Accordingly, fill the top half with 1s and save updating it
                // for last.
                self.0.write_at(u32::MAX, high_addr);

                let low = value as u32;
                self.0.write_at(low, low_addr);

                // At this point the timer could have fired if
                // 0xffff_ffff_low <= mtime, but in that case it would have
                // fired anyway if high had also been in place. It would also be
                // strange to have high != 0
                // with mtimecmp so small.

                let high = (value >> 32) as u32;
                self.0.write_at(high, high_addr);
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(target_arch = "riscv64")] {
        type TimeIo = Mmio<u64>;
        type TimerIo = Mmio<u64>;
    } else {
        type TimeIo = MtimeIo;
        type TimerIo = MtimecmpIo;
    }
}

struct Mtimer {
    time_io: TimeIo,
    timer_io: TimerIo,
}

impl Mtimer {
    fn new(time_base: usize, timecmp_base: usize) -> Self {
        Self {
            time_io: TimeIo::new(time_base, size_of::<Mtime>()),
            timer_io: TimerIo::new(timecmp_base, TIMECMP_APERTURE_SIZE),
        }
    }
}

pub fn read_time() -> u64 {
    *Mtime::read_from(&get_timer().time_io)
}

#[allow(unused)]
fn set(time: u64) {
    Mtimecmp::from(time).write_nth_to(
        &get_timer().timer_io,
        arch::riscv::current_cpu_number() as usize,
    );
}
