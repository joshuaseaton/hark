// Copyright (c) 2025 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

mod sbi;
pub use sbi::*;

use core::{arch::asm, ptr};

use bitfld::{bitfield_repr, layout};
use derive_more::{Deref, From};

cfg_if::cfg_if! {
    if #[cfg(any(doc, target_arch = "riscv32", target_arch = "riscv64"))] {
        #[doc(inline)]
        pub use crate::sbi_call;

        use regio::riscv::csr;

        use crate::{CallFrame, ArchCommon};
    }
}

layout!({
    /// `misa`: Machine ISA Register
    #[csr(misa)]
    pub struct Misa(usize);
    {
        #[cfg(target_pointer_width = "64")]
        {
            let _: Bits<61, 26> = 0;
        }
        #[cfg(target_pointer_width = "32")]
        {
            let _: Bits<28, 26> = 0;
        }
        let z: Bit<25> = 0; // Reserved
        let y: Bit<24> = 0; // Reserved
        let x: Bit<23>; // Non-standard extensions
        let w: Bit<22> = 0; // Reserved
        let v: Bit<21>; // Vector
        let u: Bit<20>; // User mode
        let t: Bit<19> = 0; // Reserved
        let s: Bit<18>; // Supervisor mode
        let r: Bit<17> = 0; // Reserved
        let q: Bit<16>; // Quad-precision floating point
        let p: Bit<15>; // Tenatively reserved for Packed-SIMD
        let o: Bit<14> = 0; // Reserved
        let n: Bit<13>; // Tentatively reserved for user-level interrupts
        let m: Bit<12>; // Integer multiply/divide
        let l: Bit<11> = 0; // Reserved
        let k: Bit<10> = 0; // Reserved
        let j: Bit<9> = 0; // Reserved
        let i: Bit<8>; // RV32I/64I base ISA
        let h: Bit<7>; // Hypervisor
        let g: Bit<6> = 0; // Reserved
        let f: Bit<5>; // Single-precision floating-point
        let e: Bit<4>; // RV32E/64E base ISA
        let d: Bit<3>; // Double-precision floating-point
        let c: Bit<2>; // Compressed
        let b: Bit<1>; // B
        let a: Bit<0>; // Atomic
    }
});

layout!({
    /// `mvendorid`: Machine Vendor ID Register
    #[csr(mvendorid, ro)]
    pub struct Mvendorid(u32);
    {
        let bank: Bits<31, 7>;
        let offset: Bits<6, 0>;
    }
});

/// `marchid`: Machine Architecture ID Register.
#[csr(marchid, ro)]
#[derive(Clone, Copy, Debug, Deref, From)]
pub struct Marchid(usize);

/// `mimpid`: Machine Implementation ID Register.
#[csr(mimpid, ro)]
#[derive(Clone, Copy, Debug, Deref, From)]
pub struct Mimpid(usize);

/// `mhartid`: Hart ID Register
#[csr(mhartid, ro)]
#[derive(Clone, Copy, Debug, Deref, From)]
pub struct Mhartid(usize);

#[bitfield_repr(u8)]
pub enum MtvecMode {
    // All traps set pc to BASE.
    Direct = 0,

    // Asynchronous interrupts set pc to BASE + (4 × cause).
    Vectored = 1,
}

layout!({
    /// `mtvec`: Machine Trap-Vector Base-Address Register
    #[csr(mtvec)]
    pub struct Mtvec(usize);
    {
        #[cfg(target_pointer_width = "64")]
        {
            let base: Bits<63, 2>;
        }
        #[cfg(target_pointer_width = "32")]
        {
            let base: Bits<31, 2>;
        }
        let mode: Bits<1, 0, MtvecMode>;
    }
});

#[bitfield_repr(u8)]
pub enum Xlen {
    _32 = 1,
    _64 = 2,
    _128 = 3,
}

#[bitfield_repr(u8)]
pub enum StatusExtState {
    Off = 0,     // Off (FS, VS); All off (XS)
    Initial = 1, // Initial (FS, VS); None dirty or clean, some on (XS)
    Clean = 2,   // Clean (FS, VS); None dirty, some clean (XS)
    Dirty = 3,   // Dirty (FS, VS); Some dirty (XS)
}

layout!({
    /// `sstatus`: Supervisor Status Register.
    #[csr(sstatus, ro)]
    pub struct Sstatus(usize);
    {
        #[cfg(target_pointer_width = "64")]
        {
            let sd: Bit<63>; // State Dirty
            let _: Bits<62, 34>;
            let uxl: Bits<33, 32, Xlen>; // UXLEN
            let _: Bit<31>;
        }
        #[cfg(target_pointer_width = "32")]
        {
            let sd: Bit<31>;
        }

        let _: Bits<30, 25>;
        let sdt: Bit<24>;
        let spelp: Bit<23>;
        let _: Bits<22, 20>;
        let mxr: Bit<19>; // Make eXecutable Readable
        let sum: Bit<18>; // Supervisor User Memory
        let _: Bit<17>;
        let xs: Bits<16, 15, StatusExtState>; // other eXtension State
        let fs: Bits<14, 13, StatusExtState>; // F extension State
        let _: Bits<12, 11>;
        let vs: Bits<10, 9, StatusExtState>; // V extension State
        let spp: Bit<8>; // Supervisor Previous Privilege
        let _: Bit<7>;
        let ube: Bit<6>; // User Big-Endian
        let spie: Bit<5>; // Supervisor Previous Interrupt Enable
        let _: Bits<4, 3>;
        let sie: Bit<1>; // Supervisor Interrupt Enable
        let _: Bit<0>;
    }
});

cfg_if::cfg_if! {
    if #[cfg(any(doc, target_arch = "riscv32", target_arch = "riscv64"))] {
        pub(super) struct Arch {}

        impl ArchCommon for Arch {

            #[inline(always)]
            fn frame_pointer() -> usize {
                let mut fp: usize;
                unsafe {
                    asm!("mv {}, s0", out(reg) fp);
                }
                fp
            }

            fn call_frame(fp: usize) -> CallFrame {
                unsafe {
                    let frame: *const usize = ptr::without_provenance(fp);
                    CallFrame{
                        frame_pointer: *frame.sub(2),
                        return_address: *frame.sub(1)
                    }
                }
            }
        }
    }
}
