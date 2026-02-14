// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT
#![no_std]

use core::marker::PhantomData;
use core::ops::Deref;
use core::ptr;

use zerocopy::{FromBytes, IntoBytes};

#[doc(inline)]
pub use regio_macro::offset;

/// An abstract means of register access.
pub trait IoBackend {
    /// The underlying value type for register reads and writes.
    type Base;

    /// The type used to address registers.
    type Addr;

    /// Describes the mode of access. One of [`ReadOnly`], [`ReadWrite`],
    /// or [`WriteOnly`] is expected.
    type Mode: IoMode;

    /// Read a value at the given address.
    fn read_at(&self, addr: Self::Addr) -> Self::Base
    where
        Self::Mode: Readable;

    /// Write a value to the given address.
    fn write_at(&self, value: Self::Base, addr: Self::Addr)
    where
        Self::Mode: Writable;
}

/// Marker trait for I/O access modes.
pub trait IoMode {}

/// Marker for I/O modes permitting reads.
pub trait Readable: IoMode {}

/// Marker for I/O modes permitting writes.
pub trait Writable: IoMode {}

/// A tag type describing a backend as having read-only access.
///
/// Intended to be used as an instantiation of [`IoBackend::Mode`].
pub struct ReadOnly {}
impl IoMode for ReadOnly {}
impl Readable for ReadOnly {}

/// A tag type describing a backend as having read and write access.
///
/// Intended to be used as an instantiation of [`IoBackend::Mode`].
pub struct ReadWrite {}
impl IoMode for ReadWrite {}
impl Readable for ReadWrite {}
impl Writable for ReadWrite {}

/// A tag type describing a backend as having write-only access.
///
/// Intended to be used as an instantiation of [`IoBackend::Mode`].
pub struct WriteOnly {}
impl IoMode for WriteOnly {}
impl Writable for WriteOnly {}

/// A register type, with flexibly abstracted I/O and addressing.
pub trait Register: From<Self::Base> + Deref<Target = Self::Base> + Addr {
    type Base: Copy;

    /// Only enabled if the register has an unfixed address, this method reads
    /// the value at a given address from any I/O backend supporting read
    /// access.
    #[inline]
    fn read_from_at<Io>(io: &Io, addr: Self::Addr) -> Self
    where
        Self: UnfixedAddr,
        Io: IoBackend<Base = Self::Base, Addr = Self::Addr>,
        Io::Mode: Readable,
    {
        Self::from(io.read_at(addr))
    }

    /// Only enabled if the register has a fixed address, this method reads
    /// the value at the register's address from any I/O backend supporting
    /// read access.
    #[inline]
    fn read_from<Io>(io: &Io) -> Self
    where
        Self: FixedAddr,
        Io: IoBackend<Base = Self::Base, Addr = Self::Addr>,
        Io::Mode: Readable,
    {
        Self::from(io.read_at(Self::ADDR))
    }

    /// Only enabled if the register has an unfixed address and a fixed I/O
    /// backend supporting reads, this method reads the value at a given
    /// address from its backend.
    #[inline]
    fn read_at(addr: Self::Addr) -> Self
    where
        Self: UnfixedAddr,
        Self: FixedIo<Self::Base, Self::Addr>,
        <<Self as FixedIo<Self::Base, Self::Addr>>::Io as IoBackend>::Mode: Readable,
    {
        Self::from(Self::Io::default().read_at(addr))
    }

    /// Only enabled if the register has a fixed address and a fixed I/O
    /// backend supporting reads, this method reads the value at the
    /// register's address from its backend.
    #[inline]
    fn read() -> Self
    where
        Self: FixedAddr,
        Self: FixedIo<Self::Base, Self::Addr>,
        <<Self as FixedIo<Self::Base, Self::Addr>>::Io as IoBackend>::Mode: Readable,
    {
        Self::from(Self::Io::default().read_at(Self::ADDR))
    }

    /// Only enabled if the register has an unfixed address, this method
    /// writes the value at a given address to any I/O backend supporting
    /// write access.
    #[inline]
    fn write_to_at<Io>(self, io: &Io, addr: Self::Addr)
    where
        Self: UnfixedAddr,
        Io: IoBackend<Base = Self::Base, Addr = Self::Addr>,
        Io::Mode: Writable,
    {
        io.write_at(*self, addr)
    }

    /// Only enabled if the register has a fixed address, this method writes
    /// the value at the register's address to any I/O backend supporting
    /// write access.
    #[inline]
    fn write_to<Io>(self, io: &Io)
    where
        Self: FixedAddr,
        Io: IoBackend<Base = Self::Base, Addr = Self::Addr>,
        Io::Mode: Writable,
    {
        io.write_at(*self, Self::ADDR)
    }

    /// Only enabled if the register has an unfixed address and a fixed I/O
    /// backend supporting writes, this method writes the value at a given
    /// address to its backend.
    #[inline]
    fn write_at(self, addr: Self::Addr)
    where
        Self: UnfixedAddr,
        Self: FixedIo<Self::Base, Self::Addr>,
        <<Self as FixedIo<Self::Base, Self::Addr>>::Io as IoBackend>::Mode: Writable,
    {
        Self::Io::default().write_at(*self, addr)
    }

    /// Only enabled if the register has a fixed address and a fixed I/O
    /// backend supporting writes, this method writes the value at the
    /// register's address to its backend.
    #[inline]
    fn write(self)
    where
        Self: FixedAddr,
        Self: FixedIo<Self::Base, Self::Addr>,
        <<Self as FixedIo<Self::Base, Self::Addr>>::Io as IoBackend>::Mode: Writable,
    {
        Self::Io::default().write_at(*self, Self::ADDR)
    }

    /// Only enabled if the register has an unfixed address, this method
    /// performs a read-modify-write at a given address on any I/O backend
    /// supporting read and write access.
    #[inline]
    fn modify_with_at<Io, F>(io: &Io, mutate: F, addr: Self::Addr)
    where
        Self: UnfixedAddr,
        Io: IoBackend<Base = Self::Base, Addr = Self::Addr>,
        Io::Mode: Readable + Writable,
        F: FnOnce(&mut Self),
    {
        let mut value = Register::read_from_at(io, addr);
        mutate(&mut value);
        value.write_to_at(io, addr)
    }

    /// Only enabled if the register has a fixed address, this method performs
    /// a read-modify-write at the register's address on any I/O backend
    /// supporting read and write access.
    #[inline]
    fn modify_with<Io, F>(io: &Io, mutate: F)
    where
        Self: FixedAddr,
        Io: IoBackend<Base = Self::Base, Addr = Self::Addr>,
        Io::Mode: Readable + Writable,
        F: FnOnce(&mut Self),
    {
        let mut value = Register::read_from(io);
        mutate(&mut value);
        value.write_to(io)
    }

    /// Only enabled if the register has an unfixed address and a fixed I/O
    /// backend supporting reads and writes, this method performs a
    /// read-modify-write at a given address on its backend.
    #[inline]
    fn modify_at<F>(mutate: F, addr: Self::Addr)
    where
        Self: UnfixedAddr,
        Self: FixedIo<Self::Base, Self::Addr>,
        <<Self as FixedIo<Self::Base, Self::Addr>>::Io as IoBackend>::Mode: Readable + Writable,
        F: FnOnce(&mut Self),
    {
        let mut value = Register::read_at(addr);
        mutate(&mut value);
        value.write_at(addr)
    }

    /// Only enabled if the register has a fixed address and a fixed I/O
    /// backend supporting reads and writes, this method performs a
    /// read-modify-write at the register's address on its backend.
    #[inline]
    fn modify<F>(mutate: F)
    where
        Self: FixedAddr,
        Self: FixedIo<Self::Base, Self::Addr>,
        <<Self as FixedIo<Self::Base, Self::Addr>>::Io as IoBackend>::Mode: Readable + Writable,
        F: FnOnce(&mut Self),
    {
        let mut value = Register::read();
        mutate(&mut value);
        value.write()
    }
}

impl<T, Base> Register for T
where
    T: From<Base> + Deref<Target = Base> + Addr,
    Base: Copy,
{
    type Base = Base;
}

/// An abstract register addressing scheme, which need not represent a
/// conventional address type.
pub trait Addr {
    /// The address type.
    type Addr: Copy;
}

/// A register with a fixed, known address.
pub trait FixedAddr: Addr {
    const ADDR: Self::Addr;
}

/// A register with no fixed address.
pub trait UnfixedAddr: Addr {}

/// Associates a register with a default [`IoBackend`].
pub trait FixedIo<Base, Addr> {
    type Io: Default + IoBackend<Base = Base, Addr = Addr>;
}

/// An offset into an MMIO aperture at some context-defined stride.
#[derive(Clone, Copy, Debug)]
pub struct Offset(pub usize);

impl Deref for Offset {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

mod sealed {
    pub trait Sealed {}
}

/// Represents a register value type that can be widened to and truncated from a
/// type representing a wider bus width.
///
/// Implemented for all pairs of unsigned integer types up to u64 where
/// `Self` is no wider than `Access`.
///
/// This trait is sealed and cannot be implemented outside of this crate.
pub trait FitsIn<Access>: sealed::Sealed {
    fn widen(self) -> Access;
    fn truncate(access: Access) -> Self;
}

impl sealed::Sealed for u8 {}
impl sealed::Sealed for u16 {}
impl sealed::Sealed for u32 {}
impl sealed::Sealed for u64 {}

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

/// Represents an aperture of memory-mapped I/O.
///
/// Implements an [`IoBackend`] parameterized by the logical register width
/// (`Reg`) and the physical bus width (`Access`, defaulting to `Reg`),
/// addressed by register offset ([`Offset`]).
///
/// ```ignore
/// use regio::Mmio;
///
/// // An 8-bit register space at 0x1000'0000 spanning 8 bytes.
/// let io = Mmio::<u8>::new(0x1000_0000, 8);
///
/// // 8-bit registers accessed at 0x1000'0000 over a 32-bit bus, spanning
/// // sizeof(u32) * 8 bytes.
/// let io = Mmio::<u8, u32>::new(0x1000_0000, 8);
/// ```
pub struct Mmio<Reg, Access = Reg>
where
    Reg: FitsIn<Access>,
    Access: FromBytes + IntoBytes,
{
    base: *mut Access,
    size: usize,
    phantom: PhantomData<Reg>,
}

impl<Reg, Access> Mmio<Reg, Access>
where
    Reg: FitsIn<Access>,
    Access: FromBytes + IntoBytes,
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

impl<Reg, Access> IoBackend for Mmio<Reg, Access>
where
    Reg: FitsIn<Access>,
    Access: FromBytes + IntoBytes,
{
    type Addr = Offset;
    type Base = Reg;
    type Mode = ReadWrite;

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
