// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::cmp::min;
use core::fmt;
use core::marker::PhantomData;

use bitfld::{bitfield_repr, layout};
use regio::{IoBackend, Mmio, Offset, Readable, Register as _, Writable, offset};

use crate::dev::uart::DriverBase;

const FIFO_DEPTH_16750: usize = 64;
const FIFO_DEPTH_16550A: usize = 16;

const APERTURE_SIZE: usize = 8;

// When DLAB = 0, a read from the first register yields a received byte.
// TODO: blocks?
layout! {{
    #[offset(0)]
    struct RxBufferRegister(u8);
    {
        let data: Bits<7, 0>;
    }
}}

// When DLAB = 0, a write to the first register transmits the data.
layout! {{
    #[offset(0)]
    struct TxBufferRegister(u8);
    {
        let data: Bits<7, 0>;
    }
}}

// When DLAB = 0.
layout! {{
    #[offset(1)]
    struct InterruptEnableRegister(u8);
    {
        let _: Bits<7, 4> = 0;
        let modem_status: Bit<3>;
        let line_status: Bit<2>;
        let tx_empty: Bit<1>;
        let rx_available: Bit<0>;
    }
}}

// When DLAB = 1.
layout! {{
    #[offset(0)]
    struct DivisorLatchLowerRegister(u8);
    {
        let data: Bits<7, 0>;
    }
}}

// When DLAB = 1.
layout! {{
    #[offset(1)]
    struct DivisorLatchUpperRegister(u8);
    {
        let data: Bits<7, 0>;
    }
}}

layout! {{
    #[offset(2)]
    struct FifoControlRegister(u8);
    {
        let receiver_trigger: Bits<7, 6>;
        let extended_fifo_enable: Bit<5>;
        let _: Bit<4> = 0;
        let dma_mode: Bit<3>;
        let tx_fifo_reset: Bit<2>;
        let rx_fifo_reset: Bit<1>;
        let fifo_enable: Bit<0>;
    }
}}

impl FifoControlRegister {
    const MAX_TRIGGER_LEVEL: u8 = 0b11;
}

#[bitfield_repr(u8)]
enum InterruptType {
    ModemStatus = 0b0000,
    None = 0b0001,
    TxEmpty = 0b0010,
    RxDataAvailable = 0b0100,
    RxLineStatus = 0b0110,
    CharTimeout = 0b1100,
}

layout! {{
    #[offset(2)]
    struct InterruptIdentRegister(u8);
    {
        let fifos_enabled: Bits<7, 6>;
        let extended_fifo_enabled: Bit<5>;
        let _: Bit<4> = 0;
        let interrupt_id: Bits<3, 0, InterruptType>;
    }
}}

#[bitfield_repr(u8)]
enum LineControlWordLength {
    Five = 0b00,
    Six = 0b01,
    Seven = 0b10,
    Eight = 0b11,
}

layout! {{
    #[offset(3)]
    struct LineControlRegister(u8);
    {
        let divisor_latch_access: Bit<7>; // The "DLAB"
        let break_control: Bit<6>;
        let stick_parity: Bit<5>;
        let even_parity: Bit<4>;
        let parity_enable: Bit<3>;
        let stop_bits: Bit<2>;
        let word_length: Bits<1, 0, LineControlWordLength>;
    }
}}

impl LineControlRegister {
    const STOP_BITS_ONE: bool = false;
    const STOP_BITS_TWO: bool = true;
}

layout! {{
    #[offset(4)]
    struct ModemControlRegister(u8);
    {
        let _: Bits<7, 6>;
        let automatic_flow_control_enable: Bit<5>;
        let loopback: Bit<4>;
        let auxiliary_out_2: Bit<3>;
        let auxiliary_out_1: Bit<2>;
        let request_to_send: Bit<1>;
        let data_terminal_ready: Bit<0>;
    }
}}

layout! {{
    #[offset(5)]
    struct LineStatusRegister(u8);
    {
        let error_in_rx_fifo: Bit<7>;
        let tx_empty: Bit<6>;
        let tx_register_empty: Bit<5>;
        let break_interrupt: Bit<4>;
        let framing_error: Bit<3>;
        let parity_error: Bit<2>;
        let overrun_error: Bit<1>;
        let data_ready: Bit<0>;
    }
}}

layout! {{
    #[offset(6)]
    struct ModemStatusRegister(u8);
    {
        let data_carrier_detect: Bit<7>;
        let ring_indicator: Bit<6>;
        let data_set_ready: Bit<5>;
        let clear_to_send: Bit<4>;
        let delta_data_carrier_detect: Bit<3>;
        let trailing_edge_ring_indicator: Bit<2>;
        let delta_data_set_ready: Bit<1>;
        let delta_clear_to_send: Bit<0>;
    }
}}

layout! {{
    #[offset(7)]
    struct ScratchRegister(u8);
    {
        let data: Bits<7, 0>;
    }
}}

#[derive(Default)]
pub struct State {
    fifo_depth: usize,
}

pub trait UartIo: IoBackend<Base = u8, Addr = Offset, Mode: Readable + Writable> {
    const DESC: &str;
    fn new(addr: usize, size: usize) -> Self;
}

impl UartIo for Mmio<u8> {
    const DESC: &str = "8-bit MMIO access";
    fn new(addr: usize, size: usize) -> Self {
        Self::new(addr, size)
    }
}

pub struct Base<Io: UartIo>(PhantomData<Io>);

impl<Io: UartIo> DriverBase for Base<Io> {
    type Io = Io;
    type State = State;

    fn io(base: usize) -> Io {
        Io::new(base, APERTURE_SIZE)
    }

    fn describe(w: &mut impl fmt::Write, state: &State) {
        write!(
            w,
            "ns8250 UART ({}, FIFO depth = {})",
            Io::DESC,
            state.fifo_depth
        )
        .unwrap();
    }

    fn init(io: &Io, state: &mut State) {
        // Disable all interrupts.
        InterruptEnableRegister::from(0).write_to(io);

        LineControlRegister::from(0)
            .set_word_length(LineControlWordLength::Eight) // Deal in 8 bits
            .set_stop_bits(LineControlRegister::STOP_BITS_ONE) // One is faster
            .set_divisor_latch_access(true) // Enable the latch
            .write_to(io);

        // Extended FIFO mode must be enabled while the divisor latch is.
        // Be sure to preserve the line controls, modulo divisor latch access,
        // which should be disabled immediately after configuring the FIFO.
        FifoControlRegister::from(0)
            .set_fifo_enable(true)
            .set_extended_fifo_enable(true)
            .set_rx_fifo_reset(true)
            .set_tx_fifo_reset(true)
            .set_receiver_trigger(FifoControlRegister::MAX_TRIGGER_LEVEL)
            .write_to(io);

        // With the FIFO configured we commit the divisor by clearing the latch.
        LineControlRegister::modify_with(io, |reg| {
            reg.set_divisor_latch_access(false);
        });

        // Drive flow control bits high since we don't actively manage them.
        ModemControlRegister::from(0)
            .set_data_terminal_ready(true)
            .set_request_to_send(true)
            .write_to(io);

        // Figure out the FIFO depth.
        let iir = InterruptIdentRegister::read_from(io);
        state.fifo_depth = if iir.fifos_enabled() == 0 {
            // A disabled FIFO is a FIFO of length 1 for all intents and
            // purposes.
            1
        } else if iir.extended_fifo_enabled() {
            FIFO_DEPTH_16750
        } else {
            FIFO_DEPTH_16550A
        };
    }

    fn tx_ready(io: &Io) -> bool {
        LineStatusRegister::read_from(io).tx_register_empty()
    }

    fn write<'a>(io: &Io, state: &State, bytes: &'a [u8]) -> &'a [u8] {
        // We assume this FIFO is empty when this method is called.
        let count = min(state.fifo_depth, bytes.len());
        let (bytes, rest) = unsafe { bytes.split_at_unchecked(count) };
        for byte in bytes {
            TxBufferRegister::from(*byte).write_to(io);
        }
        rest
    }
}
