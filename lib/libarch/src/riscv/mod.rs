// Copyright (c) 2025 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

mod sbi;
pub use sbi::*;

#[doc(inline)]
#[cfg(any(doc, target_arch = "riscv32", target_arch = "riscv64"))]
pub use crate::sbi_call;

use bitfld::{bitfield_repr, layout};

#[bitfield_repr(u8)]
pub enum Xlen {
    _32 = 1,
    _64 = 2,
    _128 = 3,
}

#[bitfield_repr(u8)]
pub enum SstatusExtState {
    Off = 0,     // Off (FS, VS); All off (XS)
    Initial = 1, // Initial (FS, VS); None dirty or clean, some on (XS)
    Clean = 2,   // Clean (FS, VS); None dirty, some clean (XS)
    Dirty = 3,   // Dirty (FS, VS); Some dirty (XS)
}

// Bits omitted here are all WPRI (Reserved Writes Preserve Values, Reads
// Ignore Values): only the exact values read from them should ever be
// written back; writing other values may have no effect or may be invalid.
layout!({
    /// The Supervisor Status Register.
    pub struct Sstatus(u64);
    {
        let sd: Bit<63>; // State Dirty
        let _: Bits<62, 34>;
        let uxl: Bits<33, 32, Xlen>; // UXLEN
        let _: Bits<31, 20>;
        let mxr: Bit<19>; // Make eXecutable Readable
        let sum: Bit<18>; // Supervisor User Memory
        let _: Bit<17>;
        let xs: Bits<16, 15, SstatusExtState>; // other eXtension State
        let fs: Bits<14, 13, SstatusExtState>; // F extension State
        let _: Bits<12, 11>;
        let vs: Bits<10, 9, SstatusExtState>; // V extension State
        let spp: Bit<8>; // Supervisor Previous Privilege
        let _: Bit<7>;
        let ube: Bit<6>; // User Big-Endian
        let spie: Bit<5>; // Supervisor Previous Interrupt Enable
        let _: Bits<4, 3>;
        let sie: Bit<1>; // Supervisor Interrupt Enable
        let _: Bit<0>;
    }
});
