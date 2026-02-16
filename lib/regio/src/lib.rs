// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT
#![no_std]

pub mod riscv;

use core::marker::PhantomData;
use core::ops::Deref;
use core::ptr;

use zerocopy::{FromBytes, IntoBytes};

/// Associates a type as being a register addressed at a fixed offset.
///
/// Requires that the type implements `core::ops::Deref`. Implements
/// `regio::Spec` with
///   * `Base = <Self as core::ops::Deref>::Target`
///   * `Addr = regio::Offset`;
///   * and `Access` as given by the second parameter, defaulting to
///     `regio::ReadWrite`
///
/// ## Parameters
///
/// Comma-separated and positional:
///
///   - *Required:* the register offset as a `usize` expression.
///     <br><br>
///   - *Optional:* one of `ro`, `rw`, or `wo`, corresponding to
///     `regio::{ReadOnly, ReadWrite, WriteOnly}`, respectively.
///
///     *Default:* `rw`
///
pub use regio_macro::offset;

/// An abstract means of register access.
pub trait IoBackend {
    /// The underlying value type for register reads and writes.
    type Base;

    /// The type used to address registers.
    type Addr;

    /// Read a value at the given address.
    fn read_at(&self, addr: Self::Addr) -> Self::Base;

    /// Write a value to the given address.
    fn write_at(&self, value: Self::Base, addr: Self::Addr);
}

/// A marker trait for registers permitting reads.
pub trait Readable: AccessMode {}

/// A marker trait for registers permitting writes.
pub trait Writable: AccessMode {}

/// A tag type describing a register as being read-only.
pub struct ReadOnly {}
impl AccessMode for ReadOnly {}
impl Readable for ReadOnly {}

/// A tag type describing a register as being readable and writable.
pub struct ReadWrite {}
impl AccessMode for ReadWrite {}
impl Readable for ReadWrite {}
impl Writable for ReadWrite {}

/// A tag type describing a register as being write-only.
pub struct WriteOnly {}
impl AccessMode for WriteOnly {}
impl Writable for WriteOnly {}

/// An abstract register specification.
pub trait Spec {
    /// The underlying type of the register (expected to be an unsigned integral
    /// type).
    type Base: Copy;

    /// An abstract register addressing scheme, which need not represent a
    /// conventional address type.
    type Addr: Copy;

    /// The access modes permitted by the associated register. Bounded by an
    /// internal trait, this type must be one of [`ReadOnly`], [`ReadWrite`],
    /// or [`WriteOnly`].
    type Access: AccessMode;
}

/// A register with a fixed, known address.
pub trait FixedAddr: Spec {
    const ADDR: Self::Addr;
}

/// A register with no fixed address.
pub trait UnfixedAddr: Spec {}

/// A register with a default (and default-constructible) I/O backend.
pub trait DefaultIo: Spec {
    type Io: Default + IoBackend<Base = Self::Base, Addr = Self::Addr>;
}

/// A register type, with flexibly abstracted I/O and addressing.
pub trait Register: Spec + From<Self::Base> + Deref<Target = Self::Base> {
    /// Only enabled if the register is readable with an unfixed address, this
    /// method reads the register value from any compatible backend at a given
    /// address
    #[inline]
    fn read_from_at<Io>(io: &Io, addr: Self::Addr) -> Self
    where
        Self: UnfixedAddr + Spec<Access: Readable>,
        Io: IoBackend<Base = Self::Base, Addr = Self::Addr>,
    {
        Self::from(io.read_at(addr))
    }

    /// Only enabled if the register is readable with a fixed address, this
    /// method reads the register value from any compatible I/O backend at its
    /// associated address.
    #[inline]
    fn read_from<Io>(io: &Io) -> Self
    where
        Self: FixedAddr + Spec<Access: Readable>,
        Io: IoBackend<Base = Self::Base, Addr = Self::Addr>,
    {
        Self::from(io.read_at(Self::ADDR))
    }

    /// Only enabled if the register is readable with an unfixed address and a
    /// I/O default backend, this reads the register from its associated backend
    /// at the given address.
    #[inline]
    fn read_at(addr: Self::Addr) -> Self
    where
        Self: UnfixedAddr + DefaultIo + Spec<Access: Readable>,
    {
        Self::from(Self::Io::default().read_at(addr))
    }

    /// Only enabled if the register is redable with a fixed address and a
    /// default I/O backend, this method reads the register value from its
    /// associated backend at its associated address.
    #[inline]
    fn read() -> Self
    where
        Self: FixedAddr + DefaultIo + Spec<Access: Readable>,
    {
        Self::from(Self::Io::default().read_at(Self::ADDR))
    }

    /// Only enabled if the register is writable with an unfixed address, this
    /// method writes the register value to any compatible I/O backend at a
    /// given address.
    #[inline]
    fn write_to_at<Io>(self, io: &Io, addr: Self::Addr)
    where
        Self: UnfixedAddr + Spec<Access: Writable>,
        Io: IoBackend<Base = Self::Base, Addr = Self::Addr>,
    {
        io.write_at(*self, addr)
    }

    /// Only enabled if the register is writable with a fixed address, this
    /// method writes the register value to any compatible I/O backend at its
    /// associated address.
    #[inline]
    fn write_to<Io>(self, io: &Io)
    where
        Self: FixedAddr + Spec<Access: Writable>,
        Io: IoBackend<Base = Self::Base, Addr = Self::Addr>,
    {
        io.write_at(*self, Self::ADDR)
    }

    /// Only enabled if the register is writable with an unfixed address and a
    /// default I/O backend, this method writes the register value to the
    /// associated backend at a given address.
    #[inline]
    fn write_at(self, addr: Self::Addr)
    where
        Self: UnfixedAddr + DefaultIo + Spec<Access: Writable>,
    {
        Self::Io::default().write_at(*self, addr)
    }

    /// Only enabled if the register is writable with a fixed address and a
    /// default I/O backend, this method writes the register value to the
    /// associated backend at the associated address.
    #[inline]
    fn write(self)
    where
        Self: FixedAddr + DefaultIo + Spec<Access: Writable>,
    {
        Self::Io::default().write_at(*self, Self::ADDR)
    }

    /// Only enabled if the register is readable and writable with an unfixed
    /// address, this method performs a read-modify-write with on any compatible
    /// I/O backend at a given address.
    #[inline]
    fn modify_with_at<Io, F>(io: &Io, mutate: F, addr: Self::Addr)
    where
        Self: UnfixedAddr + Spec<Access: Readable + Writable>,
        Io: IoBackend<Base = Self::Base, Addr = Self::Addr>,
        F: FnOnce(&mut Self),
    {
        let mut value = Register::read_from_at(io, addr);
        mutate(&mut value);
        value.write_to_at(io, addr)
    }

    /// Only enabled if the register is readable and writable with an fixed
    /// address, this method performs a read-modify-write with on any compatible
    /// I/O backend at its associatedaddress.
    #[inline]
    fn modify_with<Io, F>(io: &Io, mutate: F)
    where
        Self: FixedAddr + Spec<Access: Readable + Writable>,
        Io: IoBackend<Base = Self::Base, Addr = Self::Addr>,
        F: FnOnce(&mut Self),
    {
        let mut value = Register::read_from(io);
        mutate(&mut value);
        value.write_to(io)
    }

    /// Only enabled if the register is readable and writable with an unfixed
    /// address and a default I/O backend, this method performs a
    /// read-modify-write with its associated backend at a given address.
    #[inline]
    fn modify_at<F>(mutate: F, addr: Self::Addr)
    where
        Self: UnfixedAddr + DefaultIo + Spec<Access: Readable + Writable>,
        F: FnOnce(&mut Self),
    {
        let mut value = Register::read_at(addr);
        mutate(&mut value);
        value.write_at(addr)
    }

    /// Only enabled if the register has a fixed address and a fixed I/O
    /// backend supporting reads and writes, this method performs a
    /// read-modify-write at the register's address on its backend.
    ///
    /// Only enabled if the register is readable and writable with a fixed
    /// address and a default I/O backend, this method performs a
    /// read-modify-write with its associated backend at its associated address.
    #[inline]
    fn modify<F>(mutate: F)
    where
        Self: FixedAddr + DefaultIo + Spec<Access: Readable + Writable>,
        F: FnOnce(&mut Self),
    {
        let mut value = Register::read();
        mutate(&mut value);
        value.write()
    }
}

impl<T> Register for T where T: Spec + From<Self::Base> + Deref<Target = Self::Base> {}

/// An offset into an MMIO aperture at some context-defined stride.
#[derive(Clone, Copy, Debug)]
pub struct Offset(pub usize);

impl Deref for Offset {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

mod internal {
    // A marker trait for I/O access modes.
    pub trait AccessMode {}

    pub trait FitsIn<Access> {
        fn widen(self) -> Access;
        fn truncate(access: Access) -> Self;
    }

    macro_rules! fits_in {
        ($narrow:ty, $wide:ty) => {
            impl FitsIn<$wide> for $narrow {
                fn widen(self) -> $wide {
                    self as $wide
                }
                fn truncate(wide: $wide) -> Self {
                    wide as Self
                }
            }
        };
    }

    fits_in!(u8, u8);
    fits_in!(u8, u16);
    fits_in!(u8, u32);
    fits_in!(u8, u64);
    fits_in!(u16, u16);
    fits_in!(u16, u32);
    fits_in!(u16, u64);
    fits_in!(u32, u32);
    fits_in!(u32, u64);
    fits_in!(u64, u64);
}
use internal::AccessMode;

/// Represents an aperture of memory-mapped I/O.
///
/// Implements an [`IoBackend`] parameterized by the logical register width
/// (`Reg`) and the physical bus width (`Access`, defaulting to `Reg`),
/// addressed by register offset ([`Offset`]).
///
/// ```
/// use regio::Mmio;
///
/// // An 8-bit register space at 0x1000'0000 spanning 8 bytes.
/// let io = Mmio::<u8>::new(0x1000_0000, 8);
///
/// // 8-bit registers accessed at 0x1000'0000 over a 32-bit bus, spanning
/// // sizeof(u32) * 8 bytes.
/// let io = Mmio::<u8, u32>::new(0x1000_0000, 8);
/// ```
///
/// `Reg` is bounded by an internal trait representing a register value type
/// that can be widened to and truncated from a type representing a wider bus
/// width. This trait is implemented for for all pairs of unsigned integer types
/// up to u64 where `Self` is no wider than `Access`.
pub struct Mmio<Reg, AccessType = Reg>
where
    Reg: internal::FitsIn<AccessType>,
    AccessType: FromBytes + IntoBytes,
{
    base: *mut AccessType,
    size: usize,
    phantom: PhantomData<Reg>,
}

impl<Reg, AccessType> Mmio<Reg, AccessType>
where
    Reg: internal::FitsIn<AccessType>,
    AccessType: FromBytes + IntoBytes,
{
    /// Creates an MMIO backend over the given aperture.
    pub fn new(base: usize, size: usize) -> Self {
        Self {
            base: ptr::without_provenance_mut(base),
            size,
            phantom: PhantomData {},
        }
    }
}

impl<Reg, AccessType> IoBackend for Mmio<Reg, AccessType>
where
    Reg: internal::FitsIn<AccessType>,
    AccessType: FromBytes + IntoBytes,
{
    type Addr = Offset;
    type Base = Reg;

    fn read_at(&self, offset: Offset) -> Reg {
        debug_assert!(*offset < self.size);
        unsafe {
            let ptr = self.base.add(*offset);
            Reg::truncate(ptr::read_volatile(ptr))
        }
    }

    fn write_at(&self, value: Reg, offset: Offset) {
        debug_assert!(*offset < self.size);
        unsafe {
            let ptr = self.base.add(*offset);
            ptr::write_volatile(ptr, value.widen())
        }
    }
}
