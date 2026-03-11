// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::arch::asm;
use core::error;
use core::fmt;
use core::num::NonZeroIsize;

use bitfld::layout;
use derive_more::{Deref, From};

/// Represents an SBI call error.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Deref, From, Eq, PartialEq)]
pub struct Error(NonZeroIsize);

impl Error {
    /// `SBI_ERR_FAILED`
    pub const FAILED: Self = Self(NonZeroIsize::new(-1).unwrap());
    /// `SBI_ERR_NOT_SUPPORTED`
    pub const NOT_SUPPORTED: Self = Self(NonZeroIsize::new(-2).unwrap());
    /// `SBI_ERR_INVALID_PARAM`
    pub const INVALID_PARAM: Self = Self(NonZeroIsize::new(-3).unwrap());
    /// `SBI_ERR_DENIED`
    pub const DENIED: Self = Self(NonZeroIsize::new(-4).unwrap());
    /// `SBI_ERR_INVALID_ADDRESS`
    pub const INVALID_ADDRESS: Self = Self(NonZeroIsize::new(-5).unwrap());
    /// `SBI_ERR_ALREADY_AVAILABLE`
    pub const ALREADY_AVAILABLE: Self = Self(NonZeroIsize::new(-6).unwrap());
    /// `SBI_ERR_ALREADY_STARTED`
    pub const ALREADY_STARTED: Self = Self(NonZeroIsize::new(-7).unwrap());
    /// `SBI_ERR_ALREADY_STOPPED`
    pub const ALREADY_STOPPED: Self = Self(NonZeroIsize::new(-8).unwrap());
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Error::FAILED => write!(f, "failed"),
            Error::NOT_SUPPORTED => write!(f, "not supported"),
            Error::INVALID_PARAM => write!(f, "invalid parameter"),
            Error::DENIED => write!(f, "denied"),
            Error::INVALID_ADDRESS => write!(f, "invalid address"),
            Error::ALREADY_AVAILABLE => write!(f, "already available"),
            Error::ALREADY_STARTED => write!(f, "already started"),
            Error::ALREADY_STOPPED => write!(f, "already stopped"),
            _ => write!(f, "unknown SBI error ({:#x})", self.get()),
        }
    }
}

impl error::Error for Error {}

/// Represents an SBI implementation ID.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Deref, Eq, PartialEq)]
pub struct ImplementationId(usize);

impl ImplementationId {
    pub const BERKELEY_BOOT_LOADER: usize = 0;
    pub const OPEN_SBI: usize = 1;
    pub const XVISOR: usize = 2;
    pub const KVM: usize = 3;
    pub const RUST_SBI: usize = 4;
    pub const DIOSIX: usize = 5;
    pub const COFFER: usize = 6;
}

impl fmt::Display for ImplementationId {
    /// Gives the name of the implementer if known.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match **self {
            ImplementationId::BERKELEY_BOOT_LOADER => write!(f, "Berkeley Boot Loader (BBL)"),
            ImplementationId::OPEN_SBI => write!(f, "OpenSBI"),
            ImplementationId::XVISOR => write!(f, "Xvisor"),
            ImplementationId::KVM => write!(f, "KVM"),
            ImplementationId::RUST_SBI => write!(f, "Rust SBI"),
            ImplementationId::DIOSIX => write!(f, "Diosix"),
            ImplementationId::COFFER => write!(f, "Coffer"),
            other => write!(f, "unknown SBI implementation ({other:#x})"),
        }
    }
}

layout!({
    /// Represents an SBI specification version.
    pub struct SpecificationVersion(u32);
    {
        let _: Bit<31> = 0;
        let major: Bits<30, 24>;
        let minor: Bits<23, 0>;
    }
});

impl fmt::Display for SpecificationVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major(), self.minor())
    }
}

// A separate helper macro for call!() internals so that no internal arms
// are documented by rustdoc.
#[doc(hidden)]
#[macro_export]
#[cfg(any(doc, target_arch = "riscv32", target_arch = "riscv64"))]
macro_rules! call_internal {
    (@asm $eid:expr, $fid:expr, $error:ident, $value:ident) => {
        #[allow(clippy::macro_metavars_in_unsafe)]
        unsafe {
            asm!(
                "ecall",
                in("a7") i32::from($eid),
                in("a6") i32::from($fid),
                lateout("a0") $error,
                lateout("a1") $value,
            )
        }
    };
    (@asm $eid:expr, $fid:expr, $error:ident, $value:ident, $a0:expr) => {
        #[allow(clippy::macro_metavars_in_unsafe)]
        unsafe {
            asm!(
                "ecall",
                in("a7") i32::from($eid),
                in("a6") i32::from($fid),
                inlateout("a0") usize::from($a0) => $error,
                lateout("a1") $value,
            )
        }
    };
    (@asm $eid:expr, $fid:expr, $error:ident, $value:ident, $a0:expr, $a1:expr $(, $a2:expr $(, $a3:expr $(, $a4:expr $(, $a5:expr)?)?)?)?) => {
        #[allow(clippy::macro_metavars_in_unsafe)]
        unsafe {
            asm!(
                "ecall",
                in("a7") i32::from($eid),
                in("a6") i32::from($fid),
                inlateout("a0") usize::from($a0) => $error,
                inlateout("a1") usize::from($a1) => $value,
                $(in("a2") usize::from($a2),
                $(in("a3") usize::from($a3),
                $(in("a4") usize::from($a4),
                $(in("a5") usize::from($a5),)?)?)?)?
            )
        }
    };
}

/// Makes an SBI call with a variadic number of arguments (up to 6), returning
/// the value if successful:
/// ```text
/// sbi::call!(eid: i32, fid: i32 [, arg: usize, ...]) -> Result<usize, sbi::Error>
/// ```
#[doc(hidden)] // Re-exported in riscv module
#[macro_export]
#[cfg(any(doc, target_arch = "riscv32", target_arch = "riscv64"))]
macro_rules! call {
    ($eid:expr, $fid:expr $(, $($args:expr),*)?) => {{
        let error: isize;
        let value: isize;
        call_internal!(@asm $eid, $fid, error, value $($(, $args)*)?);
        if let Some(error) = NonZeroIsize::new(error) {
            Err(Error(error))
        } else {
            Ok(value.cast_unsigned())
        }
    }};
}

#[doc(inline)]
pub use crate::call;

cfg_if::cfg_if! {
    if #[cfg(any(doc, target_arch = "riscv32", target_arch = "riscv64"))] {
        //
        // Base extension functions
        //

        const EID_BASE_EXTENSION: i32 = 0x10;

        /// Returns the SBI specification version.
        pub fn get_spec_version() -> SpecificationVersion {
            let value = call!(EID_BASE_EXTENSION, 0)
                .expect("sbi::get_spec_version() failed, contrary to the spec");
            #[allow(clippy::cast_possible_truncation)]
            SpecificationVersion::from(value as u32)
        }

        /// Returns the SBI implementation ID.
        ///
        /// # Errors
        ///
        /// Undocumented.
        pub fn get_impl_id() -> Result<ImplementationId, Error> {
            let value = call!(EID_BASE_EXTENSION, 1)?;
            Ok(ImplementationId(value))
        }

        /// Returns the SBI implementation version.
        ///
        /// # Errors
        ///
        /// Undocumented.
        pub fn get_impl_version() -> Result<usize, Error> {
            call!(EID_BASE_EXTENSION, 2)
        }

        /// Returns whether a given SBI extension is available.
        ///
        /// # Errors
        ///
        /// Undocumented.
        pub fn probe_extension(eid: i32) -> Result<bool, Error> {
            call!(EID_BASE_EXTENSION, 3, eid.cast_unsigned() as usize).map(|value| value > 0)
        }

        /// Returns the machine vendor ID.
        ///
        /// # Errors
        ///
        /// Undocumented.
        pub fn get_mvendorid() -> Result<usize, Error> {
            call!(EID_BASE_EXTENSION, 4)
        }

        /// Returns the machine architecture ID.
        ///
        /// # Errors
        ///
        /// Undocumented.
        pub fn get_marchid() -> Result<usize, Error> {
            call!(EID_BASE_EXTENSION, 5)
        }

        /// Returns the machine implementation ID.
        ///
        /// # Errors
        ///
        /// Undocumented.
        pub fn get_mimpid() -> Result<usize, Error> {
            call!(EID_BASE_EXTENSION, 6)
        }

        //
        // Debug console functions.
        //
        // Spec'd at https://github.com/riscv-non-isa/riscv-sbi-doc/blob/master/src/ext-debug-console.adoc
        //

        const EID_DBCN_EXTENSION: i32 = 0x4442_434e;

        /// Writes bytes to the SBI debug console.
        ///
        /// # Errors
        ///
        /// * [`Error::DENIED`]: Writes to the debug console are not allowed
        /// * [`Error::FAILED`]: Failed to write due to I/O errors
        pub fn debug_console_write(bytes: &[u8]) -> Result<usize, Error> {
            call!(
                EID_DBCN_EXTENSION,
                0,
                bytes.len(),
                bytes.as_ptr().addr(),
                0usize
            )
        }

        /// Reads bytes from the SBI debug console.
        ///
        /// # Errors
        ///
        /// * [`Error::DENIED`]: Reads to the debug console are not allowed.
        /// * [`Error::FAILED`]: Failed to read due to I/O errors
        pub fn debug_console_read(bytes: &mut [u8]) -> Result<usize, Error> {
            call!(
                EID_DBCN_EXTENSION,
                1,
                bytes.len(),
                bytes.as_ptr().addr(),
                0usize
            )
        }

        /// Writes one byte to the SBI debug console.
        ///
        /// # Errors
        ///
        /// * [`Error::DENIED`]: Writes to the debug console are not allowed
        /// * [`Error::FAILED`]: Failed to write due to I/O errors
        pub fn debug_console_write_byte(byte: u8) -> Result<(), Error> {
            call!(EID_DBCN_EXTENSION, 2, byte).map(|_| ())
        }

        const EID_TIME_EXTENSION: i32 = 0x5449_4d45;

        /// Sets a timer interrupt to fire at an absolute time.
        ///
        /// # Errors
        ///
        /// Undocumented.
        pub fn set_timer(stime_value: u64) -> Result<(), Error> {
            if cfg!(target_arch = "riscv32") {
                let lo = stime_value as usize;
                let hi = (stime_value >> 32) as usize;
                call!(EID_TIME_EXTENSION, 0, lo, hi).map(|_| ())
            } else {
                call!(EID_TIME_EXTENSION, 0, stime_value as usize).map(|_| ())
            }
        }
    }
}
