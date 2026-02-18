// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

mod riscv;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::{DeriveInput, Expr, Ident, Token, parse_macro_input};

#[proc_macro_attribute]
pub fn offset(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let ty = &input.ident;
    let OffsetAttrs { offset, access } = parse_macro_input!(attr as OffsetAttrs);
    let marker_impls = access.marker_impls(ty);
    quote! {
        #input

        impl ::regio::Register for #ty {
            type Base = <Self as ::core::ops::Deref>::Target;
            type Addr = ::regio::Offset;
        }

        impl ::regio::FixedAddr for #ty {
            const ADDR: ::regio::Offset = ::regio::Offset(#offset);
        }

        #marker_impls
    }
    .into()
}

#[proc_macro_attribute]
pub fn riscv_csr(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as riscv::CsrAttrs);
    let input = parse_macro_input!(item as DeriveInput);
    attrs.expand(input).into()
}

struct OffsetAttrs {
    offset: Expr,
    access: AccessMode,
}

impl Parse for OffsetAttrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let offset: Expr = input.parse()?;
        let access: AccessMode = if input.is_empty() {
            AccessMode::ReadWrite
        } else {
            let _: Token![,] = input.parse()?;
            input.parse()?
        };
        Ok(Self { offset, access })
    }
}

enum AccessMode {
    ReadOnly,
    ReadWrite,
    WriteOnly,
}
impl Parse for AccessMode {
    fn parse(input: ParseStream) -> Result<Self> {
        let ident: Ident = input.parse()?;
        if ident == "ro" {
            Ok(AccessMode::ReadOnly)
        } else if ident == "rw" {
            Ok(AccessMode::ReadWrite)
        } else if ident == "wo" {
            Ok(AccessMode::WriteOnly)
        } else {
            Err(Error::new_spanned(
                ident,
                "access mode must be one of `ro`, `rw`, or `wo`, or \
                unspecified altogether for a default of `rw`",
            ))
        }
    }
}

impl AccessMode {
    pub(crate) fn marker_impls(&self, ty: &Ident) -> TokenStream2 {
        match self {
            AccessMode::ReadOnly => quote! {
                impl ::regio::Readable for #ty {}
            },
            AccessMode::ReadWrite => quote! {
                impl ::regio::Readable for #ty {}
                impl ::regio::Writable for #ty {}
            },
            AccessMode::WriteOnly => quote! {
                impl ::regio::Writable for #ty {}
            },
        }
    }
}
