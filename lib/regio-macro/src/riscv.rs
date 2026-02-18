// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream, Result};
use syn::{DeriveInput, Ident, LitStr, Token};

use crate::AccessMode;

pub(super) struct CsrAttrs {
    csr: Ident,
    access: AccessMode,
}

impl Parse for CsrAttrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let csr: Ident = input.parse()?;
        let access = if input.is_empty() {
            AccessMode::ReadWrite
        } else {
            let _: Token![,] = input.parse()?;
            input.parse()?
        };
        Ok(Self { csr, access })
    }
}

impl CsrAttrs {
    pub(crate) fn expand(self, input: DeriveInput) -> TokenStream {
        let CsrAttrs { csr, access } = self;
        let type_name = &input.ident;
        let vis = &input.vis;
        let io = format_ident!("{type_name}Io");
        let csrr = LitStr::new(&format!("csrr {{}}, {csr}"), csr.span());
        let csrw = LitStr::new(&format!("csrw {csr}, {{}}"), csr.span());
        let csrrw = LitStr::new(&format!("csrrw {{}}, {csr}, {{}}"), csr.span());
        let csrrs = LitStr::new(&format!("csrrs {{}}, {csr}, {{}}"), csr.span());
        let csrrc = LitStr::new(&format!("csrrc {{}}, {csr}, {{}}"), csr.span());

        let io_doc = match access {
            AccessMode::ReadOnly => format!("Reads `{csr}`."),
            AccessMode::ReadWrite => format!("Reads/writes `{csr}`."),
            AccessMode::WriteOnly => format!("Writes `{csr}`."),
        };

        let marker_impls = access.marker_impls(type_name);
        quote! {
            #input

            impl ::regio::Register for #type_name {
                type Base = <#type_name as ::core::ops::Deref>::Target;
                type Addr = ();
            }

            impl ::regio::FixedAddr for #type_name {
                const ADDR: () = ();
            }

            #marker_impls

            impl ::regio::DefaultIo for #type_name {
                #[doc = #io_doc]
                type Io = #io;
            }

            #[doc(hidden)]
            #[derive(Default)]
            #vis struct #io;

            #[cfg(any(doc, target_arch = "riscv32", target_arch = "riscv64"))]
            impl ::regio::IoBackend for #io {
                type Base = <#type_name as ::core::ops::Deref>::Target;
                type Addr = ();

                #[inline]
                fn read_at(&self, _: ()) -> Self::Base {
                    let value: Self::Base;
                    unsafe {
                        ::core::arch::asm!(
                            #csrr,
                            out(reg) value,
                            options(nomem, nostack, preserves_flags),
                        )
                    }
                    value.into()
                }

                #[inline]
                fn write_at(&self, value: Self::Base, _: ()) {
                    unsafe {
                        ::core::arch::asm!(
                            #csrw,
                            in(reg) value,
                            options(nomem, nostack, preserves_flags),
                        )
                    }
                }
            }

            impl ::regio::AtomicIoBackend for #io {
                fn atomic_swap_at(&self, value: Self::Base, _: ()) -> Self::Base {
                    let initial: Self::Base;
                    unsafe {
                        ::core::arch::asm!(
                            #csrrw,
                            out(reg) initial,
                            in(reg) value,
                            options(nomem, nostack, preserves_flags),
                        )
                    }
                    initial
                }

                fn atomic_set_bits_at(&self, value: Self::Base, _: ()) -> Self::Base {
                    let initial: Self::Base;
                    unsafe {
                        ::core::arch::asm!(
                            #csrrs,
                            out(reg) initial,
                            in(reg) value,
                            options(nomem, nostack, preserves_flags),
                        )
                    }
                    initial
                }

                fn atomic_clear_bits_at(&self, value: Self::Base, _: ()) -> Self::Base {
                    let initial: Self::Base;
                    unsafe {
                        ::core::arch::asm!(
                            #csrrc,
                            out(reg) initial,
                            in(reg) value,
                            options(nomem, nostack, preserves_flags),
                        )
                    }
                    initial
                }
            }
        }
    }
}
