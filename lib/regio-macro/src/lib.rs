// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Lit, parse_macro_input};

/// Associates the annotated type as having a fixed register offset.
///
/// This implements `Addr<Addr = Offset>` and `FixedAddr`.
///
/// ## Parameters
///   * the register offset as a `usize`
#[proc_macro_attribute]
pub fn offset(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let ty = &input.ident;
    let offset = parse_macro_input!(attr as Lit);
    quote! {
        #input

        impl ::regio::Addr for #ty {
            type Addr = ::regio::Offset;
        }

        impl ::regio::FixedAddr for #ty {
            const ADDR: ::regio::Offset = ::regio::Offset(#offset);
        }
    }
    .into()
}
