// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::fmt;

use bitfld::layout;
use regio::{Mmio, Register as _, offset};

use super::DriverBase;

const FIFO_SIZE: usize = 8;

layout!({
    #[offset(0)]
    struct TransmitDataRegister(u32);
    {
        let full: Bit<31>; // Whether the TX FIFO is full.
        let _: Bits<30, 8>;
        let data: Bits<7, 0>;
    }
});

layout!({
    #[offset(4)]
    struct ReceiveDataRegister(u32);
    {
        let empty: Bit<31>; // Whether the RX FIFO is empty.
        let _: Bits<30, 8>;
        let data: Bits<7, 0>;
    }
});

layout!({
    #[offset(8)]
    struct TransmitControlRegister(u32);
    {
        let _: Bits<31, 19>;
        let txcnt: Bits<18, 16>; // Transmit watermark level
        let _: Bits<15, 2>;
        let nstop: Bit<1>; // Number of stop bits
        let txen: Bit<0>; // Transmit enable
    }
});

impl TransmitControlRegister {
    const STOP_BITS_ONE: bool = false;
    const STOP_BITS_TWO: bool = true;
}

layout!({
    #[offset(0xc)]
    struct ReceiveControlRegister(u32);
    {
        let _: Bits<31, 19>;
        let rxcnt: Bits<18, 16>; // Receive watermark level
        let _: Bits<15, 1>;
        let rxen: Bit<0>; // Receive enable
    }
});

layout!({
    #[offset(0x10)]
    struct InterruptEnableRegister(u32);
    {
        let _: Bits<31, 2>;
        let rxwm: Bit<1>; // Receive watermark interrupt enable
        let txwm: Bit<0>; // Transmit watermark interrupt enable
    }
});

layout!({
    #[offset(0x14, ro)]
    struct InterruptPendingRegister(u32);
    {
        let _: Bits<31, 2>;
        let rxwm: Bit<1>; // Receive watermark interrupt pending
        let txwm: Bit<0>; // Transmit watermark interrupt pending
    }
});

layout!({
    #[offset(0x18)]
    struct BaudRateDivisorRegister(u32);
    {
        let _: Bits<31, 16>;
        let div: Bits<15, 0>; // Baud rate divisor
    }
});

pub struct Base {}

impl DriverBase for Base {
    type Io = Mmio<u32>;

    type State = ();

    fn io(base: usize) -> Self::Io {
        Mmio::new(base, 0x1c)
    }

    fn describe(w: &mut impl fmt::Write, (): &Self::State) {
        let _ = write!(w, "SiFive UART");
    }

    fn init(io: &Self::Io, (): &mut Self::State) {
        // Ensure interrupts are disabled.
        InterruptEnableRegister::from(0).write_to(io);

        TransmitControlRegister::from(0)
            .set_txcnt(1) // Set ip.txwm when the TX FIFO is empty
            .set_nstop(TransmitControlRegister::STOP_BITS_ONE) // One is faster.
            .set_txen(true)
            .write_to(io);

        ReceiveControlRegister::from(0)
            .set_rxcnt(0) // Set ip.rxwm when the RX FIFO is non-empty
            .set_rxen(true)
            .write_to(io);
    }

    fn tx_fifo_is_empty(io: &Self::Io) -> bool {
        // We set txcnt to 1, so this should only be true when empty.
        InterruptPendingRegister::read_from(io).txwm()
    }

    fn read_byte(io: &Self::Io) -> Option<u8> {
        let rx = ReceiveDataRegister::read_from(io);
        (!rx.empty()).then(|| rx.data())
    }

    fn fill_empty_tx_fifo(
        io: &Self::Io,
        (): &Self::State,
        bytes: &mut impl Iterator<Item = u8>,
    ) -> bool {
        let mut space = FIFO_SIZE;
        while space > 0 {
            let Some(byte) = bytes.next() else {
                return false;
            };
            TransmitDataRegister::from(0).set_data(byte).write_to(io);
            space -= 1;
        }
        true
    }
}
