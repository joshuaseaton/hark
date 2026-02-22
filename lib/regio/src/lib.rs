// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT
#![no_std]

pub mod riscv;

use core::marker::PhantomData;
use core::ops::{Add, Deref};
use core::ptr;

use derive_more::{Deref, From};
use zerocopy::{FromBytes, IntoBytes};

/// Associates a type as being a register addressed at a fixed offset.
///
/// Requires that the type implements `core::ops::Deref` and
/// `From<<Self as core::ops::Deref>::Target>`. Implements
/// [`Register`] (with `Base = <Self as core::ops::Deref>::Target` and
/// `Addr = regio::Offset`), [`FixedAddr`], and the appropriate access marker
/// traits ([`Readable`] and/or [`Writable`]).
///
/// ## Parameters
///
/// Comma-separated and positional:
///
///   - *Required:* the register offset as a `usize` expression.
///     <br><br>
///   - *Optional:* one of `ro`, `rw`, or `wo`, indicating read-only,
///     read-write, or write-only access, respectively.
///
///     *Default:* `rw`
///
pub use regio_macro::offset;

/// Associates a type as being an arrayed register at a fixed base offset.
///
/// Like [`offset`], but additionally implements [`Arrayed`].
///
/// ## Parameters
///
///   - *Required:* the base register offset as a `usize` expression.
///     <br><br>
///   - *Optional:* one of `ro`, `rw`, or `wo`, indicating read-only,
///     read-write, or write-only access, respectively.
///
///     *Default:* `rw`
///     <br><br>
///   - *Optional:* `stride = <usize expression>`, the address increment
///     between consecutive instances.
///
///     *Default:* `size_of::<<Self as Deref>::Target>()`
///
/// The access mode and stride may be specified in either order after the
/// offset.
///
pub use regio_macro::array;

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

/// An [`IoBackend`] extension supporting atomic read-modify-write operations.
pub trait AtomicIoBackend: IoBackend {
    /// Atomically swap the value at the given address, returning the original.
    fn atomic_swap_at(&self, value: Self::Base, addr: Self::Addr) -> Self::Base;

    /// Atomically set the indicated bits at the given address, returning the
    /// original value.
    fn atomic_set_bits_at(&self, bits: Self::Base, addr: Self::Addr) -> Self::Base;

    /// Atomically clear the indicated bits at the given address, returning the
    /// original value.
    fn atomic_clear_bits_at(&self, bits: Self::Base, addr: Self::Addr) -> Self::Base;
}

/// A marker trait for registers permitting reads.
pub trait Readable {}

/// A marker trait for registers permitting writes.
pub trait Writable {}

/// A register with a fixed, known address.
pub trait FixedAddr: Register {
    const ADDR: Self::Addr;
}

/// A register with no fixed address.
pub trait UnfixedAddr: Register {}

/// A register with a default (and default-constructible) I/O backend.
pub trait DefaultIo: Register {
    type Io: Default + IoBackend<Base = Self::Base, Addr = Self::Addr>;
}

/// A register type that may appear across multiple, evenly-spaced addresses.
///
/// This requires that one may "add" `usize`s to the register's abstract
/// address type.
pub trait Arrayed: Register<Addr: Add<usize, Output = Self::Addr>> {
    /// The address increment between consecutive elements.
    ///
    /// Defaults to the size of the register (representing a contiguous array).
    const STRIDE: usize = size_of::<Self>();
}

/// A register type, with flexibly abstracted I/O and addressing.
pub trait Register: From<Self::Base> + Deref<Target = Self::Base> {
    /// The underlying type of the register (expected to be an unsigned integral
    /// type).
    type Base: Copy;

    /// An abstract register addressing scheme, which need not represent a
    /// conventional address in memory.
    type Addr: Copy;

    /// Only enabled if the register is readable with an unfixed address, this
    /// method reads the register value from any compatible backend at a given
    /// address.
    #[inline]
    fn read_from_at<Io>(io: &Io, addr: Self::Addr) -> Self
    where
        Self: Readable + UnfixedAddr,
        Io: IoBackend<Base = Self::Base, Addr = Self::Addr>,
    {
        Self::from(io.read_at(addr))
    }

    /// Like [`read_from_at`](Self::read_from_at), but reads the `n`th
    /// arrayed instance.
    #[inline]
    fn read_nth_from_at<Io>(io: &Io, n: usize, addr: Self::Addr) -> Self
    where
        Self: Readable + Arrayed + UnfixedAddr,
        Io: IoBackend<Base = Self::Base, Addr = Self::Addr>,
    {
        Self::from(io.read_at(addr + n * Self::STRIDE))
    }

    /// Only enabled if the register is readable with a fixed address, this
    /// method reads the register value from any compatible I/O backend at its
    /// associated address.
    #[inline]
    fn read_from<Io>(io: &Io) -> Self
    where
        Self: Readable + FixedAddr,
        Io: IoBackend<Base = Self::Base, Addr = Self::Addr>,
    {
        Self::from(io.read_at(Self::ADDR))
    }

    /// Like [`read_from`](Self::read_from), but reads the `n`th arrayed
    /// instance.
    #[inline]
    fn read_nth_from<Io>(io: &Io, n: usize) -> Self
    where
        Self: Readable + Arrayed + FixedAddr,
        Io: IoBackend<Base = Self::Base, Addr = Self::Addr>,
    {
        Self::from(io.read_at(Self::ADDR + n * Self::STRIDE))
    }

    /// Only enabled if the register is readable with an unfixed address and a
    /// default I/O backend, this reads the register from its associated backend
    /// at the given address.
    #[inline]
    fn read_at(addr: Self::Addr) -> Self
    where
        Self: Readable + UnfixedAddr + DefaultIo,
    {
        Self::from(Self::Io::default().read_at(addr))
    }

    /// Like [`read_at`](Self::read_at), but reads the `n`th arrayed instance.
    #[inline]
    fn read_nth_at(n: usize, addr: Self::Addr) -> Self
    where
        Self: Readable + Arrayed + UnfixedAddr + DefaultIo,
    {
        Self::from(Self::Io::default().read_at(addr + n * Self::STRIDE))
    }

    /// Only enabled if the register is readable with a fixed address and a
    /// default I/O backend, this method reads the register value from its
    /// associated backend at its associated address.
    #[inline]
    fn read() -> Self
    where
        Self: Readable + FixedAddr + DefaultIo,
    {
        Self::from(Self::Io::default().read_at(Self::ADDR))
    }

    /// Like [`read`](Self::read), but reads the `n`th arrayed instance.
    #[inline]
    fn read_nth(n: usize) -> Self
    where
        Self: Readable + Arrayed + FixedAddr + DefaultIo,
    {
        Self::from(Self::Io::default().read_at(Self::ADDR + n * Self::STRIDE))
    }

    /// Only enabled if the register is writable with an unfixed address, this
    /// method writes the register value to any compatible I/O backend at a
    /// given address.
    #[inline]
    fn write_to_at<Io>(self, io: &Io, addr: Self::Addr)
    where
        Self: Writable + UnfixedAddr,
        Io: IoBackend<Base = Self::Base, Addr = Self::Addr>,
    {
        io.write_at(*self, addr)
    }

    /// Like [`write_to_at`](Self::write_to_at), but writes to the `n`th
    /// arrayed instance.
    #[inline]
    fn write_nth_to_at<Io>(self, io: &Io, n: usize, addr: Self::Addr)
    where
        Self: Writable + Arrayed + UnfixedAddr,
        Io: IoBackend<Base = Self::Base, Addr = Self::Addr>,
    {
        io.write_at(*self, addr + n * Self::STRIDE)
    }

    /// Only enabled if the register is writable with a fixed address, this
    /// method writes the register value to any compatible I/O backend at its
    /// associated address.
    #[inline]
    fn write_to<Io>(self, io: &Io)
    where
        Self: Writable + FixedAddr,
        Io: IoBackend<Base = Self::Base, Addr = Self::Addr>,
    {
        io.write_at(*self, Self::ADDR)
    }

    /// Like [`write_to`](Self::write_to), but writes to the `n`th arrayed
    /// instance.
    #[inline]
    fn write_nth_to<Io>(self, io: &Io, n: usize)
    where
        Self: Writable + Arrayed + FixedAddr,
        Io: IoBackend<Base = Self::Base, Addr = Self::Addr>,
    {
        io.write_at(*self, Self::ADDR + n * Self::STRIDE)
    }

    /// Only enabled if the register is writable with an unfixed address and a
    /// default I/O backend, this method writes the register value to the
    /// associated backend at a given address.
    #[inline]
    fn write_at(self, addr: Self::Addr)
    where
        Self: Writable + UnfixedAddr + DefaultIo,
    {
        Self::Io::default().write_at(*self, addr)
    }

    /// Like [`write_at`](Self::write_at), but writes to the `n`th arrayed
    /// instance.
    #[inline]
    fn write_nth_at(self, n: usize, addr: Self::Addr)
    where
        Self: Writable + Arrayed + UnfixedAddr + DefaultIo,
    {
        Self::Io::default().write_at(*self, addr + n * Self::STRIDE)
    }

    /// Only enabled if the register is writable with a fixed address and a
    /// default I/O backend, this method writes the register value to the
    /// associated backend at the associated address.
    #[inline]
    fn write(self)
    where
        Self: Writable + FixedAddr + DefaultIo,
    {
        Self::Io::default().write_at(*self, Self::ADDR)
    }

    /// Like [`write`](Self::write), but writes to the `n`th arrayed instance.
    #[inline]
    fn write_nth(self, n: usize)
    where
        Self: Writable + Arrayed + FixedAddr + DefaultIo,
    {
        Self::Io::default().write_at(*self, Self::ADDR + n * Self::STRIDE)
    }

    /// Only enabled if the register is readable and writable with an unfixed
    /// address, this method performs a read-modify-write on any compatible
    /// I/O backend at a given address.
    #[inline]
    fn modify_with_at<Io, F>(io: &Io, mutate: F, addr: Self::Addr)
    where
        Self: Readable + Writable + UnfixedAddr,
        Io: IoBackend<Base = Self::Base, Addr = Self::Addr>,
        F: FnOnce(&mut Self),
    {
        let mut value = Self::read_from_at(io, addr);
        mutate(&mut value);
        value.write_to_at(io, addr)
    }

    /// Like [`modify_with_at`](Self::modify_with_at), but modifies the `n`th
    /// arrayed instance.
    #[inline]
    fn modify_nth_with_at<Io, F>(io: &Io, n: usize, mutate: F, addr: Self::Addr)
    where
        Self: Readable + Writable + Arrayed + UnfixedAddr,
        Io: IoBackend<Base = Self::Base, Addr = Self::Addr>,
        F: FnOnce(&mut Self),
    {
        let mut value = Self::read_from_at(io, addr + n * Self::STRIDE);
        mutate(&mut value);
        value.write_to_at(io, addr + n * Self::STRIDE)
    }

    /// Only enabled if the register is readable and writable with a fixed
    /// address, this method performs a read-modify-write on any compatible
    /// I/O backend at its associated address.
    #[inline]
    fn modify_with<Io, F>(io: &Io, mutate: F)
    where
        Self: Readable + Writable + FixedAddr,
        Io: IoBackend<Base = Self::Base, Addr = Self::Addr>,
        F: FnOnce(&mut Self),
    {
        let mut value = Self::read_from(io);
        mutate(&mut value);
        value.write_to(io)
    }

    /// Like [`modify_with`](Self::modify_with), but modifies the `n`th
    /// arrayed instance.
    #[inline]
    fn modify_nth_with<Io, F>(io: &Io, n: usize, mutate: F)
    where
        Self: Readable + Writable + Arrayed + FixedAddr,
        Io: IoBackend<Base = Self::Base, Addr = Self::Addr>,
        F: FnOnce(&mut Self),
    {
        let mut value = Self::from(io.read_at(Self::ADDR + n * Self::STRIDE));
        mutate(&mut value);
        io.write_at(*value, Self::ADDR + n * Self::STRIDE)
    }

    /// Only enabled if the register is readable and writable with an unfixed
    /// address and a default I/O backend, this method performs a
    /// read-modify-write with its associated backend at a given address.
    #[inline]
    fn modify_at<F>(mutate: F, addr: Self::Addr)
    where
        Self: Readable + Writable + UnfixedAddr + DefaultIo,
        F: FnOnce(&mut Self),
    {
        let mut value = Self::read_at(addr);
        mutate(&mut value);
        value.write_at(addr)
    }

    /// Like [`modify_at`](Self::modify_at), but modifies the `n`th arrayed
    /// instance.
    #[inline]
    fn modify_nth_at<F>(n: usize, mutate: F, addr: Self::Addr)
    where
        Self: Readable + Writable + Arrayed + UnfixedAddr + DefaultIo,
        F: FnOnce(&mut Self),
    {
        let mut value = Self::read_at(addr + n * Self::STRIDE);
        mutate(&mut value);
        value.write_at(addr + n * Self::STRIDE)
    }

    /// Only enabled if the register is readable and writable with a fixed
    /// address and a default I/O backend, this method performs a
    /// read-modify-write with its associated backend at its associated address.
    #[inline]
    fn modify<F>(mutate: F)
    where
        Self: Readable + Writable + FixedAddr + DefaultIo,
        F: FnOnce(&mut Self),
    {
        let mut value = Self::read();
        mutate(&mut value);
        value.write()
    }

    /// Like [`modify`](Self::modify), but modifies the `n`th arrayed instance.
    #[inline]
    fn modify_nth<F>(n: usize, mutate: F)
    where
        Self: Readable + Writable + Arrayed + FixedAddr + DefaultIo,
        F: FnOnce(&mut Self),
    {
        let io = Self::Io::default();
        let mut value = Self::from(io.read_at(Self::ADDR + n * Self::STRIDE));
        mutate(&mut value);
        io.write_at(*value, Self::ADDR + n * Self::STRIDE)
    }

    /// Only enabled if the register is readable and writable with an unfixed
    /// address, this method atomically swaps the current register value with a
    /// new one on any compatible atomic I/O backend at a given address,
    /// returning the original value.
    #[inline]
    fn atomic_swap_with_at<Io>(value: Self, io: &Io, addr: Self::Addr) -> Self
    where
        Self: Readable + Writable + UnfixedAddr,
        Io: IoBackend<Base = Self::Base, Addr = Self::Addr> + AtomicIoBackend,
    {
        Self::from(io.atomic_swap_at(*value, addr))
    }

    /// Only enabled if the register is readable and writable with a fixed
    /// address, this method atomically swaps the current register value with a
    /// new one on any compatible atomic I/O backend at its associated address,
    /// returning the original value.
    #[inline]
    fn atomic_swap_with<Io>(value: Self, io: &Io) -> Self
    where
        Self: Readable + Writable + FixedAddr,
        Io: IoBackend<Base = Self::Base, Addr = Self::Addr> + AtomicIoBackend,
    {
        Self::from(io.atomic_swap_at(*value, Self::ADDR))
    }

    /// Only enabled if the register is readable and writable with an unfixed
    /// address and a default atomic I/O backend, this method atomically swaps
    /// the current register value with a new one at a given address, returning
    /// the original value.
    #[inline]
    fn atomic_swap_at(value: Self, addr: Self::Addr) -> Self
    where
        Self: Readable + Writable + UnfixedAddr + DefaultIo<Io: AtomicIoBackend>,
    {
        Self::from(Self::Io::default().atomic_swap_at(*value, addr))
    }

    /// Only enabled if the register is readable and writable with a fixed
    /// address and a default atomic I/O backend, this method atomically swaps
    /// the current register value with a new one at its associated address,
    /// returning the original value.
    #[inline]
    fn atomic_swap(value: Self) -> Self
    where
        Self: Readable + Writable + FixedAddr + DefaultIo<Io: AtomicIoBackend>,
    {
        Self::from(Self::Io::default().atomic_swap_at(*value, Self::ADDR))
    }

    /// Only enabled if the register is readable and writable with an unfixed
    /// address, this method atomically sets the indicated bits in the current
    /// register value on any compatible atomic I/O backend at a given address,
    /// returning the original value.
    #[inline]
    fn atomic_set_bits_with_at<Io>(bits: Self, io: &Io, addr: Self::Addr) -> Self
    where
        Self: Readable + Writable + UnfixedAddr,
        Io: IoBackend<Base = Self::Base, Addr = Self::Addr> + AtomicIoBackend,
    {
        Self::from(io.atomic_set_bits_at(*bits, addr))
    }

    /// Only enabled if the register is readable and writable with a fixed
    /// address, this method atomically sets the indicated bits in the current
    /// register value on any compatible atomic I/O backend at its associated
    /// address, returning the original value.
    #[inline]
    fn atomic_set_bits_with<Io>(bits: Self, io: &Io) -> Self
    where
        Self: Readable + Writable + FixedAddr,
        Io: IoBackend<Base = Self::Base, Addr = Self::Addr> + AtomicIoBackend,
    {
        Self::from(io.atomic_set_bits_at(*bits, Self::ADDR))
    }

    /// Only enabled if the register is readable and writable with an unfixed
    /// address and a default atomic I/O backend, this method atomically sets
    /// the indicated bits in the current register value at a given address,
    /// returning the original value.
    #[inline]
    fn atomic_set_bits_at(bits: Self, addr: Self::Addr) -> Self
    where
        Self: Readable + Writable + UnfixedAddr + DefaultIo<Io: AtomicIoBackend>,
    {
        Self::from(Self::Io::default().atomic_set_bits_at(*bits, addr))
    }

    /// Only enabled if the register is readable and writable with a fixed
    /// address and a default atomic I/O backend, this method atomically sets
    /// the indicated bits in the current register value at its associated
    /// address, returning the original value.
    #[inline]
    fn atomic_set_bits(bits: Self) -> Self
    where
        Self: Readable + Writable + FixedAddr + DefaultIo<Io: AtomicIoBackend>,
    {
        Self::from(Self::Io::default().atomic_set_bits_at(*bits, Self::ADDR))
    }

    /// Only enabled if the register is readable and writable with an unfixed
    /// address, this method atomically clears the indicated bits in the current
    /// register value on any compatible atomic I/O backend at a given address,
    /// returning the original value.
    #[inline]
    fn atomic_clear_bits_with_at<Io>(bits: Self, io: &Io, addr: Self::Addr) -> Self
    where
        Self: Readable + Writable + UnfixedAddr,
        Io: IoBackend<Base = Self::Base, Addr = Self::Addr> + AtomicIoBackend,
    {
        Self::from(io.atomic_clear_bits_at(*bits, addr))
    }

    /// Only enabled if the register is readable and writable with a fixed
    /// address, this method atomically clears the indicated bits in the current
    /// register value on any compatible atomic I/O backend at its associated
    /// address, returning the original value.
    #[inline]
    fn atomic_clear_bits_with<Io>(bits: Self, io: &Io) -> Self
    where
        Self: Readable + Writable + FixedAddr,
        Io: IoBackend<Base = Self::Base, Addr = Self::Addr> + AtomicIoBackend,
    {
        Self::from(io.atomic_clear_bits_at(*bits, Self::ADDR))
    }

    /// Only enabled if the register is readable and writable with an unfixed
    /// address and a default atomic I/O backend, this method atomically clears
    /// the indicated bits in the current register value at a given address,
    /// returning the original value.
    #[inline]
    fn atomic_clear_bits_at(bits: Self, addr: Self::Addr) -> Self
    where
        Self: Readable + Writable + UnfixedAddr + DefaultIo<Io: AtomicIoBackend>,
    {
        Self::from(Self::Io::default().atomic_clear_bits_at(*bits, addr))
    }

    /// Only enabled if the register is readable and writable with a fixed
    /// address and a default atomic I/O backend, this method atomically clears
    /// the indicated bits in the current register value at its associated
    /// address, returning the original value.
    #[inline]
    fn atomic_clear_bits(bits: Self) -> Self
    where
        Self: Readable + Writable + FixedAddr + DefaultIo<Io: AtomicIoBackend>,
    {
        Self::from(Self::Io::default().atomic_clear_bits_at(*bits, Self::ADDR))
    }
}

/// An offset in a peripheral's register address space.
///
/// Interpretation in practice depends on the [`Mmio`] instance: a register
/// index when `Access > Reg`, a byte offset when `Reg == Access`.
#[derive(Clone, Copy, Debug, Deref, Eq, From, PartialEq)]
pub struct Offset(pub usize);

impl Add<usize> for Offset {
    type Output = Offset;

    fn add(self, rhs: usize) -> Self::Output {
        Self(*self + rhs)
    }
}

mod internal {
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
/// Represents an aperture of memory-mapped I/O.
///
/// Implements an [`IoBackend`] parameterized by the logical register width
/// (`Reg`) and the physical bus width (`Access`, defaulting to `Reg`),
/// addressed by register offset ([`Offset`]).
///
/// The base pointer is byte-addressed. Offsets are scaled by a factor of
/// `size_of::<Access>() / size_of::<Reg>()` to compute byte addresses:
///
///   `byte_address = base + offset * scale`
///
/// When `Access > Reg`, each register-width offset unit maps to one
/// `Access`-width physical slot.
///
/// | Type | Scale | Offset 1 maps to byte |
/// |---|---|---|
/// | `Mmio<u8>` | 1 | 1 |
/// | `Mmio<u8, u32>` | 4 | 4 |
/// | `Mmio<u32>` | 1 | 1 |
///
/// ```
/// use regio::Mmio;
///
/// // 8-bit registers, byte-addressed.
/// let io = Mmio::<u8>::new(0x1000_0000, 8);
///
/// // 8-bit registers over a 32-bit bus; offset 1 maps to byte 4.
/// let io = Mmio::<u8, u32>::new(0x1000_0000, 8);
///
/// // 32-bit registers, byte-addressed.
/// let io = Mmio::<u32>::new(0x1000_0000, 8);
/// ```
///
/// `Reg` is bounded by an internal trait representing a register value type
/// that can be widened to and truncated from a type representing a wider bus
/// width. This trait is implemented for all pairs of unsigned integer types
/// up to u64 where `Self` is no wider than `Access`.
pub struct Mmio<Reg, Access = Reg>
where
    Reg: internal::FitsIn<Access>,
    Access: FromBytes + IntoBytes,
{
    base: *mut u8,
    size: usize,
    phantom: PhantomData<(Reg, Access)>,
}

impl<Reg, Access> Mmio<Reg, Access>
where
    Reg: internal::FitsIn<Access>,
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
    Reg: internal::FitsIn<Access>,
    Access: FromBytes + IntoBytes,
{
    type Addr = Offset;
    type Base = Reg;

    fn read_at(&self, offset: Offset) -> Reg {
        let scale = size_of::<Access>() / size_of::<Reg>();
        debug_assert!(*offset < self.size);
        debug_assert!((*offset * scale).is_multiple_of(align_of::<Access>()));
        unsafe {
            let ptr = self.base.add(*offset * scale).cast::<Access>();
            Reg::truncate(ptr::read_volatile(ptr))
        }
    }

    fn write_at(&self, value: Reg, offset: Offset) {
        let scale = size_of::<Access>() / size_of::<Reg>();
        debug_assert!(*offset < self.size);
        debug_assert!((*offset * scale).is_multiple_of(align_of::<Access>()));
        unsafe {
            let ptr = self.base.add(*offset * scale).cast::<Access>();
            ptr::write_volatile(ptr, value.widen())
        }
    }
}
