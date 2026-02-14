// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::{DeriveInput, Expr, Ident, Token, parse_macro_input};

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
#[proc_macro_attribute]
pub fn offset(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let ty = &input.ident;
    let OffsetAttrs { offset, access } = parse_macro_input!(attr as OffsetAttrs);
    quote! {
        #input

        impl ::regio::Spec for #ty {
            type Base = <Self as ::core::ops::Deref>::Target;
            type Addr = ::regio::Offset;
            type Access = #access;
        }

        impl ::regio::FixedAddr for #ty {
            const ADDR: ::regio::Offset = ::regio::Offset(#offset);
        }
    }
    .into()
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

impl ToTokens for AccessMode {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            AccessMode::ReadOnly => quote! { ::regio::ReadOnly },
            AccessMode::ReadWrite => quote! { ::regio::ReadWrite },
            AccessMode::WriteOnly => quote! { ::regio::WriteOnly },
        }
        .to_tokens(tokens);
    }
}
