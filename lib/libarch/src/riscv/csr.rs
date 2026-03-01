// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::fmt;

use bitfld::{bitfield_repr, layout};
use derive_more::{Deref, From};
use regio::riscv::csr;

//
// Unprivileged Counter/Timers
//

/// `time`: Timer for RDTIME instruction.
#[csr(time, ro)]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Deref, From)]
pub struct Time(usize);

/// `timeh`: Upper 32 bits of `time` (RV32 only).
#[cfg_attr(target_arch = "riscv32", csr(timeh, ro))]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Deref, From)]
pub struct Timeh(u32);

//
// Supervisor Trap Setup
//

#[bitfield_repr(u8)]
pub enum StatusExtState {
    Off = 0,     // Off (FS, VS); All off (XS)
    Initial = 1, // Initial (FS, VS); None dirty or clean, some on (XS)
    Clean = 2,   // Clean (FS, VS); None dirty, some clean (XS)
    Dirty = 3,   // Dirty (FS, VS); Some dirty (XS)
}

layout!({
    /// `sstatus`: Supervisor Status Register.
    #[csr(sstatus)]
    pub struct Sstatus(usize);
    {
        #[cfg(target_pointer_width = "64")]
        {
            /// *S*tate *D*irty.
            let sd: Bit<63>;
            let _: Bits<62, 34>;
            /// *UXL*EN.
            let uxl: Bits<33, 32, Xlen>;
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
        /// *M*ake e*X*ecutable *R*eadable.
        let mxr: Bit<19>;
        /// *S*upervisor *U*ser *M*emory.
        let sum: Bit<18>;
        let _: Bit<17>;
        /// Other e*X*tension *S*tate.
        let xs: Bits<16, 15, StatusExtState>;
        /// *F* extension *S*tate.
        let fs: Bits<14, 13, StatusExtState>;
        let _: Bits<12, 11>;
        /// *V* extension *S*tate.
        let vs: Bits<10, 9, StatusExtState>;
        /// *S*upervisor *P*revious *P*rivilege.
        let spp: Bit<8>;
        let _: Bit<7>;
        /// *U*ser *B*ig-*E*ndian.
        let ube: Bit<6>;
        /// *S*upervisor *P*revious *I*nterrupt *E*nable.
        let spie: Bit<5>;
        let _: Bits<4, 3>;
        /// *S*upervisor *I*nterrupt *E*nable.
        let sie: Bit<1>;
        let _: Bit<0>;
    }
});

layout!({
    /// `sie`: Supervisor interrupt-enable register
    #[csr(sie)]
    pub struct Sie(usize);
    {
        /// *L*ocal *C*ounter *O*ver*F*low *I*nterrupt *E*nable.
        let lcofie: Bit<13>;
        /// *S*upervisor *E*xternal *I*nterrupt *E*nable.
        let seie: Bit<9>;
        /// *S*upervisor *T*imer *I*nterrupt *E*nable.
        let stie: Bit<5>;
        /// *S*upervisor *S*oftware *I*nterrupt *E*nable.
        let ssie: Bit<1>;
    }
});

#[bitfield_repr(u8)]
pub enum TrapVectorMode {
    // All traps set pc to BASE.
    Direct = 0,

    // Asynchronous interrupts set pc to BASE + (4 × cause).
    Vectored = 1,
}

layout!({
    /// `stvec`: Supervisor Trap-Vector Base-Address Register
    #[csr(stvec)]
    pub struct Stvec(usize);
    {
        #[cfg(target_pointer_width = "64")]
        {
            #[unshifted]
            let base: Bits<63, 2>;
        }
        #[cfg(target_pointer_width = "32")]
        {
            #[unshifted]
            let base: Bits<31, 2>;
        }
        let mode: Bits<1, 0, TrapVectorMode>;
    }
});

//
// Supervisor Trap Handling
//

/// `sscratch`: Supervisor Scratch Register
#[csr(sscratch)]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Deref, From)]
pub struct Sscratch(usize);

/// `sepc`: Supervisor Exception Program Counter Register
#[csr(sepc)]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Deref, From)]
pub struct Sepc(usize);

#[bitfield_repr(u8)]
pub enum Xlen {
    _32 = 1,
    _64 = 2,
    _128 = 3,
}

#[repr(transparent)]
#[derive(Debug, Deref, Eq, PartialEq)]
pub struct ExceptionCode(usize);

impl ExceptionCode {
    pub const INSTRUCTION_ADDRESS_MISALIGNED: Self = Self(0);
    pub const INSTRUCTION_ACCESS_FAULT: Self = Self(1);
    pub const ILLEGAL_INSTRUCTION: Self = Self(2);
    pub const BREAKPOINT: Self = Self(3);
    pub const LOAD_ADDRESS_MISALIGNED: Self = Self(4);
    pub const LOAD_ACCESS_FAULT: Self = Self(5);
    pub const STORE_OR_AMO_ADDRESS_MISALIGNED: Self = Self(6);
    pub const STORE_OR_AMO_ADDRESS_ACCESS_FAULT: Self = Self(7);
    pub const ENVIRONMENT_CALL_FROM_U_MODE: Self = Self(8);
    pub const ENVIRONMENT_CALL_FROM_S_MODE: Self = Self(9);
    // 10 is reserved
    pub const ENVIRONMENT_CALL_FROM_M_MODE: Self = Self(11);
    pub const INSTRUCTION_PAGE_FAULT: Self = Self(12);
    pub const LOAD_PAGE_FAULT: Self = Self(13);
    // 14 is reserved
    pub const STORE_OR_AMO_PAGE_FAULT: Self = Self(15);
    pub const DOUBLE_TRAP: Self = Self(16);
    // 17 is reserved
    pub const SOFTWARE_CHECK: Self = Self(18);
    pub const HARDWARE_ERROR: Self = Self(19);
    // 20-23 are reserved
    // 24-31 are designated
    // 32-47 are reserved
    // 48-63 are designated
    // >= 64 are reserved
}

impl fmt::Display for ExceptionCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::INSTRUCTION_ADDRESS_MISALIGNED => write!(f, "instruction address misaligned"),
            Self::INSTRUCTION_ACCESS_FAULT => write!(f, "instruction access fault"),
            Self::ILLEGAL_INSTRUCTION => write!(f, "illegal instruction"),
            Self::BREAKPOINT => write!(f, "breakpoint"),
            Self::LOAD_ADDRESS_MISALIGNED => write!(f, "load address misaligned"),
            Self::LOAD_ACCESS_FAULT => write!(f, "access fault"),
            Self::STORE_OR_AMO_ADDRESS_MISALIGNED => write!(f, "store/AMO address misaligned"),
            Self::STORE_OR_AMO_ADDRESS_ACCESS_FAULT => write!(f, "store/AMO address access fault"),
            Self::ENVIRONMENT_CALL_FROM_U_MODE => write!(f, "environment call from U-mode"),
            Self::ENVIRONMENT_CALL_FROM_S_MODE => write!(f, "environment call from S-mode"),
            Self::ENVIRONMENT_CALL_FROM_M_MODE => write!(f, "environment call from M-mode"),
            Self::INSTRUCTION_PAGE_FAULT => write!(f, "instruction page fault"),
            Self::LOAD_PAGE_FAULT => write!(f, "load page fault"),
            Self::STORE_OR_AMO_PAGE_FAULT => write!(f, "store/AMO page fault"),
            Self::DOUBLE_TRAP => write!(f, "double trap"),
            Self::SOFTWARE_CHECK => write!(f, "software check"),
            Self::HARDWARE_ERROR => write!(f, "hardware error"),
            _ => write!(f, "Unknown exception code: {}", **self),
        }
    }
}

#[repr(transparent)]
#[derive(Debug, Deref, Eq, PartialEq)]
pub struct InterruptCode(usize);

impl InterruptCode {
    // 0 is reserved
    pub const SUPERVISOR_SOFTWARE_INTERRUPT: Self = Self(1);
    // 2 is reserved
    pub const MACHINE_SOFTWARE_INTERRUPT: Self = Self(3);
    // 4 is reserved
    pub const SUPERVISOR_TIMER_INTERRUPT: Self = Self(5);
    // 6 is reserved
    pub const MACHINE_TIMER_INTERRUPT: Self = Self(7);
    // 8 is reserved
    pub const SUPERVISOR_EXTERNAL_INTERRUPT: Self = Self(9);
    // 10 is reserved
    pub const MACHINE_EXTERNAL_INTERRUPT: Self = Self(11);
    // 12 is reserved
    pub const COUNTER_OVERFLOW_INTERRUPT: Self = Self(13);
    // 14-15 is reserved
    // >= 16 is designated for platform use
}

impl fmt::Display for InterruptCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::SUPERVISOR_SOFTWARE_INTERRUPT => write!(f, "supervisor software interrupt"),
            Self::MACHINE_SOFTWARE_INTERRUPT => write!(f, "machine software interrupt"),
            Self::SUPERVISOR_TIMER_INTERRUPT => write!(f, "supervisor timer interrupt"),
            Self::MACHINE_TIMER_INTERRUPT => write!(f, "machine timer interrupt"),
            Self::SUPERVISOR_EXTERNAL_INTERRUPT => write!(f, "supervisor external interrupt"),
            Self::MACHINE_EXTERNAL_INTERRUPT => write!(f, "machine external interrupt"),
            Self::COUNTER_OVERFLOW_INTERRUPT => write!(f, "counter overflow interrupt"),
            _ => write!(f, "unknown interrupt code: {}", **self),
        }
    }
}

layout!({
    /// `scause`: Supervisor Cause Register.
    #[csr(scause)]
    pub struct Scause(usize);
    {
        #[cfg(target_pointer_width = "64")]
        {
            let interrupt: Bit<63>;
            let code: Bits<62, 0>;
        }
        #[cfg(target_pointer_width = "32")]
        {
            let interrupt: Bit<31>;
            let code: Bits<30, 0>;
        }
    }
});

impl Scause {
    #[inline]
    pub fn exception_code(self) -> ExceptionCode {
        debug_assert!(!self.interrupt());
        ExceptionCode(self.code() as usize)
    }

    #[inline]
    pub fn interrupt_code(self) -> InterruptCode {
        debug_assert!(self.interrupt());
        InterruptCode(self.code() as usize)
    }
}

/// `stval`: Supervisor Trap Value Register
#[csr(stval)]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Deref, From)]
pub struct Stval(usize);

//
// Supervisor Timer
//

/// `stimecmp`: Supervisor Timer Register
#[csr(stimecmp)]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Deref, From)]
pub struct Stimecmp(usize);

//
// Machine Information Registers
//

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
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Deref, From)]
pub struct Marchid(usize);

/// `mimpid`: Machine Implementation ID Register.
#[csr(mimpid, ro)]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Deref, From)]
pub struct Mimpid(usize);

/// `mhartid`: Hart ID Register
#[csr(mhartid, ro)]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Deref, From)]
pub struct Mhartid(usize);

//
// Machine Trap Setup
//

layout!({
    /// `mstatus`: Machine Status Register.
    #[csr(mstatus)]
    pub struct Mstatus(usize);
    {
        #[cfg(target_pointer_width = "64")]
        {
            /// *S*tate *D*irty.
            let sd: Bit<63>;
            let _: Bits<62, 43>;
            /// *M*achine *D*ouble *T*rap.
            let mdt: Bit<42>;
            /// *M*achine *P*revious *E*xpected *L*anding *P*ad.
            let mpelp: Bit<41>;
            let _: Bit<40>;
            /// *M*achine *P*revious *V*irtualization mode.
            let mpv: Bit<39>;
            /// *G*uest *V*irtual *A*ddress.
            let gva: Bit<38>;
            /// *M*achine *B*ig-*E*ndian.
            let mbe: Bit<37>;
            /// *S*upervisor *B*ig-*E*ndian.
            let sbe: Bit<36>;
            /// *SXL*EN.
            let sxl: Bits<35, 34>;
            /// *UXL*EN.
            let uxl: Bits<33, 32, Xlen>;
            let _: Bit<31>;
        }
        #[cfg(target_pointer_width = "32")]
        {
            let sd: Bit<31>;
        }

        let _: Bits<30, 25>;
        /// *S*oftware *D*ouble *T*rap.
        let sdt: Bit<24>;
        /// *S*upervisor *P*revious *E*xpected *L*anding *P*ad.
        let spelp: Bit<23>;
        /// *T*rap *SR*et.
        let tsr: Bit<22>;
        /// *T*imeout *W*ait.
        let tw: Bit<21>;
        /// *T*rap *V*irtual *M*emory.
        let tvm: Bit<20>;
        /// *M*ake e*X*ecutable *R*eadable.
        let mxr: Bit<19>;
        /// *S*upervisor *U*ser *M*emory.
        let sum: Bit<18>;
        /// *M*odify *PR*i*V*ilege.
        let mprv: Bit<17>;
        /// Other e*X*tension *S*tate.
        let xs: Bits<16, 15, StatusExtState>;
        /// *F* extension *S*tate.
        let fs: Bits<14, 13, StatusExtState>;
        /// *M*achine *P*revious *P*rivilege.
        let mpp: Bits<12, 11>;
        /// *V* extension *S*tate.
        let vs: Bits<10, 9, StatusExtState>;
        /// *S*upervisor *P*revious *P*rivilege.
        let spp: Bit<8>;
        /// *M*achine *P*revious *I*nterrupt *E*nable.
        let mpie: Bit<7>;
        /// *U*ser *B*ig-*E*ndian.
        let ube: Bit<6>;
        /// *S*upervisor *P*revious *I*nterrupt *E*nable.
        let spie: Bit<5>;
        let _: Bit<4>;
        /// *M*achine *I*nterrupt *E*nable.
        let mie: Bit<3>;
        let _: Bit<2>;
        /// *S*upervisor *I*nterrupt *E*nable.
        let sie: Bit<1>;
        let _: Bit<0>;
    }
});

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
        /// Reserved.
        let z: Bit<25> = 0;
        /// Reserved.
        let y: Bit<24> = 0;
        /// Non-standard e*X*tensions.
        let x: Bit<23>;
        /// Reserved.
        let w: Bit<22> = 0;
        /// *V*ector.
        let v: Bit<21>;
        /// *U*ser mode.
        let u: Bit<20>;
        /// Reserved.
        let t: Bit<19> = 0;
        /// *S*upervisor mode.
        let s: Bit<18>;
        /// Reserved.
        let r: Bit<17> = 0;
        /// *Q*uad-precision floating point.
        let q: Bit<16>;
        /// Tentatively reserved for *P*acked-SIMD.
        let p: Bit<15>;
        /// Reserved.
        let o: Bit<14> = 0;
        /// Tentatively reserved for user-level interrupts.
        let n: Bit<13>;
        /// Integer *m*ultiply/divide.
        let m: Bit<12>;
        /// Reserved.
        let l: Bit<11> = 0;
        /// Reserved.
        let k: Bit<10> = 0;
        /// Reserved.
        let j: Bit<9> = 0;
        /// RV32*I*/64*I* base ISA.
        let i: Bit<8>;
        /// *H*ypervisor.
        let h: Bit<7>;
        /// Reserved.
        let g: Bit<6> = 0;
        /// Single-precision *f*loating-point.
        let f: Bit<5>;
        /// RV32*E*/64*E* base ISA.
        let e: Bit<4>;
        /// *D*ouble-precision floating-point.
        let d: Bit<3>;
        /// *C*ompressed.
        let c: Bit<2>;
        /// *B*.
        let b: Bit<1>;
        /// *A*tomic.
        let a: Bit<0>;
    }
});

layout!({
    /// `mie`: Machine interrupt-enable register
    #[csr(mie)]
    pub struct Mie(usize);
    {
        let lcofie: Bit<13>; // Local Counter OverFlow Interrupt Enable
        let meie: Bit<11>; // Machine External Interrupt Enable
        let seie: Bit<9>; // Supervisor External Interrupt Enable
        let mtie: Bit<7>; // Machine Timer Interrupt Enable
        let stie: Bit<5>; // Supervisor Timer Interrupt Enable
        let msie: Bit<3>; // Machine Software Interrupt Enable
        let ssie: Bit<1>; // Supervisor Software Interrupt Enable
    }
});

layout!({
    /// `mtvec`: Machine Trap-Vector Base-Address Register
    #[csr(mtvec)]
    pub struct Mtvec(usize);
    {
        #[cfg(target_pointer_width = "64")]
        {
            #[unshifted]
            let base: Bits<63, 2>;
        }
        #[cfg(target_pointer_width = "32")]
        {
            #[unshifted]
            let base: Bits<31, 2>;
        }
        let mode: Bits<1, 0, TrapVectorMode>;
    }
});

layout!({
    /// `mstatush`: Additional Machine Status Register.
    #[cfg_attr(target_arch = "riscv32", csr(mstatush, ro))]
    pub struct Mstatush(u32);
    {
        let _: Bits<31, 11>;
        let mdt: Bit<10>;
        let mpelp: Bit<9>;
        let _: Bit<8>;
        let mpv: Bit<7>;
        let gva: Bit<6>;
        let mbe: Bit<5>;
        let sbe: Bit<4>;
        let _: Bits<3, 0>;
    }
});

//
// Machine Trap Handling
//

/// `mscratch`: Machine Scratch Register
#[csr(mscratch)]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Deref, From)]
pub struct Mscratch(usize);

/// `mepc`: Machine Exception Program Counter Register
#[csr(mepc)]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Deref, From)]
pub struct Mepc(usize);

layout!({
    /// `mcause`: Machine Cause Register.
    #[csr(mcause)]
    pub struct Mcause(usize);
    {
        #[cfg(target_pointer_width = "64")]
        {
            let interrupt: Bit<63>;
            let code: Bits<62, 0>;
        }
        #[cfg(target_pointer_width = "32")]
        {
            let interrupt: Bit<31>;
            let code: Bits<30, 0>;
        }
    }
});

impl Mcause {
    #[inline]
    pub fn exception_code(self) -> ExceptionCode {
        debug_assert!(!self.interrupt());
        ExceptionCode(self.code() as usize)
    }

    #[inline]
    pub fn interrupt_code(self) -> InterruptCode {
        debug_assert!(self.interrupt());
        InterruptCode(self.code() as usize)
    }
}

/// `mtval`: Machine Trap Value Register
#[csr(mtval)]
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Deref, From)]
pub struct Mtval(usize);
