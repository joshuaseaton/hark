// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

/// Declares a RISC-V CSR register type.
///
/// Placed on a struct implementing the [`Register`](crate::Register) bounds
/// (`Deref` and `From`), this generates the regio trait implementations
/// ([`Register`](crate::Register), [`FixedAddr`](crate::FixedAddr),
/// [`DefaultIo`](crate::DefaultIo)), the appropriate access marker traits
/// ([`Readable`](crate::Readable) and/or [`Writable`](crate::Writable)),
/// and an I/O backend using inline `csrr` and `csrw` instructions.
///
/// ## Parameters
///
/// Comma-separated and positional:
///
///   - *Required:* the CSR name (e.g., `sstatus` or `marchid`).
///     <br><br>
///   - *Optional:* one of `ro`, `rw`, or `wo`, indicating read-only,
///     read-write, or write-only access, respectively.
///
///     *Default:* `rw`
///
/// ## Example
///
/// ```text
/// #[csr(marchid, ro)]
/// #[derive(Debug, Clone, Copy)]
/// pub struct Marchid(usize);
///
/// impl Deref for Marchid { /* ... */ }
/// impl From<usize> for Marchid { /* ... */ }
/// ```
pub use regio_macro::riscv_csr as csr;
