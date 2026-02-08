// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use core::arch::asm;
use core::error;
use core::fmt;
use core::num::NonZeroIsize;
use core::ops::Deref;

use bitfld::layout;

/// Represents an SBI call error.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SbiError(NonZeroIsize);

impl SbiError {
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

impl fmt::Display for SbiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            SbiError::FAILED => write!(f, "failed"),
            SbiError::NOT_SUPPORTED => write!(f, "not supported"),
            SbiError::INVALID_PARAM => write!(f, "invalid parameter"),
            SbiError::DENIED => write!(f, "denied"),
            SbiError::INVALID_ADDRESS => write!(f, "invalid address"),
            SbiError::ALREADY_AVAILABLE => write!(f, "already available"),
            SbiError::ALREADY_STARTED => write!(f, "already started"),
            SbiError::ALREADY_STOPPED => write!(f, "already stopped"),
            _ => write!(f, "unknown SBI error ({:#x})", self.get()),
        }
    }
}

impl error::Error for SbiError {}

impl Deref for SbiError {
    type Target = NonZeroIsize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

layout!({
    /// Represents an SBI specification version.
    pub struct SbiSpecificationVersion(u32);
    {
        let _: Bit<31> = 0;
        let major: Bits<30, 24>;
        let minor: Bits<23, 0>;
    }
});

// A separate helper macro for sbi_call!() internals so that no internal arms
// are documented by rustdoc.
#[doc(hidden)]
#[macro_export]
#[cfg(any(doc, target_arch = "riscv32", target_arch = "riscv64"))]
macro_rules! sbi_call_internal {
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
/// sbi_call!(eid: i32, fid: i32 [, arg: usize, ...]) -> Result<usize, SbiError>
/// ```
#[doc(hidden)] // Re-exported in riscv module
#[macro_export]
#[cfg(any(doc, target_arch = "riscv32", target_arch = "riscv64"))]
macro_rules! sbi_call {
    ($eid:expr, $fid:expr $(, $($args:expr),*)?) => {{
        let error: isize;
        let value: isize;
        sbi_call_internal!(@asm $eid, $fid, error, value $($(, $args)*)?);
        if let Some(error) = NonZeroIsize::new(error) {
            Err(SbiError(error))
        } else {
            Ok(value.cast_unsigned())
        }
    }};
}

cfg_if::cfg_if! {
    if #[cfg(any(doc, target_arch = "riscv32", target_arch = "riscv64"))] {
        //
        // Base extension functions
        //

        const EID_BASE_EXTENSION: i32 = 0x10;

        /// Returns the SBI specification version.
        pub fn sbi_get_spec_version() -> SbiSpecificationVersion {
            let value = sbi_call!(EID_BASE_EXTENSION, 0)
                .expect("sbi_get_spec_version() failed, contrary to the spec");
            #[allow(clippy::cast_possible_truncation)]
            SbiSpecificationVersion::from(value as u32)
        }

        /// Returns the SBI implementation ID.
        ///
        /// # Errors
        ///
        /// Undocumented.
        pub fn sbi_get_impl_id() -> Result<usize, SbiError> {
            sbi_call!(EID_BASE_EXTENSION, 1)
        }

        /// Returns the SBI implementation version.
        ///
        /// # Errors
        ///
        /// Undocumented.
        pub fn sbi_get_impl_version() -> Result<usize, SbiError> {
            sbi_call!(EID_BASE_EXTENSION, 2)
        }

        /// Returns whether a given SBI extension is available.
        ///
        /// # Errors
        ///
        /// Undocumented.
        pub fn sbi_probe_extension(eid: i32) -> Result<bool, SbiError> {
            sbi_call!(EID_BASE_EXTENSION, 3, eid.cast_unsigned() as usize).map(|value| value > 0)
        }

        /// Returns the machine vendor ID.
        ///
        /// # Errors
        ///
        /// Undocumented.
        pub fn sbi_get_mvendorid() -> Result<usize, SbiError> {
            sbi_call!(EID_BASE_EXTENSION, 4)
        }

        /// Returns the machine architecture ID.
        ///
        /// # Errors
        ///
        /// Undocumented.
        pub fn sbi_get_marchid() -> Result<usize, SbiError> {
            sbi_call!(EID_BASE_EXTENSION, 5)
        }

        /// Returns the machine implementation ID.
        ///
        /// # Errors
        ///
        /// Undocumented.
        pub fn sbi_get_mimpid() -> Result<usize, SbiError> {
            sbi_call!(EID_BASE_EXTENSION, 6)
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
        /// * `SbiError::DENIED`: Writes to the debug console are not allowed
        /// * `SbiError::FAILED`: Failed to write due to I/O errors
        pub fn sbi_debug_console_write(bytes: &[u8]) -> Result<usize, SbiError> {
            sbi_call!(
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
        /// * `SbiError::DENIED`: Reads to the debug console are not allowed.
        /// * `SbiError::FAILED`: Failed to read due to I/O errors
        pub fn sbi_debug_console_read(bytes: &mut [u8]) -> Result<usize, SbiError> {
            sbi_call!(
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
        /// * `SbiError::DENIED`: Writes to the debug console are not allowed
        /// * `SbiError::FAILED`: Failed to write due to I/O errors
        pub fn sbi_debug_console_write_byte(byte: u8) -> Result<(), SbiError> {
            sbi_call!(EID_DBCN_EXTENSION, 2, byte).map(|_| ())
        }
    }
}
