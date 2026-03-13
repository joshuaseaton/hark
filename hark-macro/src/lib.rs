// Copyright (c) 2026 Joshua Seaton
//
// Use of this source code is governed by a MIT-style
// license that can be found in the LICENSE file or at
// https://opensource.org/licenses/MIT

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::parse::{self, Parse, ParseStream};
use syn::{Error, Expr, ExprLit, Ident, ItemFn, Lit, LitStr, Meta, Token, parse_macro_input};

// See docstring on the re-export in the hark crate.
#[proc_macro_attribute]
pub fn hark_test(attr: TokenStream, item: TokenStream) -> TokenStream {
    if !attr.is_empty() {
        return Error::new_spanned(
            TokenStream2::from(attr),
            "hark_test takes no attribute parameters",
        )
        .to_compile_error()
        .into();
    }

    let func = parse_macro_input!(item as ItemFn);
    let name = &func.sig.ident;
    let name_str = name.to_string();
    let const_name = format_ident!("HARK_TEST_{}", name_str.to_uppercase());

    quote! {
        #[used]
        #[unsafe(link_section = ".data.hark.tests")]
        static #const_name: ::hark::testing::TestSpec = ::hark::testing::TestSpec {
            suite: module_path!(),
            case: #name_str,
            func: #name,
        };

        const _: fn() -> Result<(), ::hark::testing::Failure> = #name;

        #func
    }
    .into()
}

// See docstring on the re-export in the hark crate.
#[proc_macro_attribute]
pub fn shell_command(attr: TokenStream, item: TokenStream) -> TokenStream {
    let help = parse_macro_input!(attr as HelpAttr);
    let help_str = &help.0;

    let cmd = parse_macro_input!(item as Command);
    let name_str = cmd.name().to_string();
    let const_name = format_ident!("HARK_COMMAND_{}", name_str.to_uppercase());
    let name = cmd.name().clone();
    let Command { func, doc } = cmd;

    quote! {
        #[used]
        #[unsafe(link_section = ".data.hark.commands")]
        static #const_name: ::hark::shell::CommandSpec = ::hark::shell::CommandSpec {
            name: #name_str,
            desc: #doc,
            help: #help_str,
            func: #name,
        };

        const _: fn(::hark::shell::Args) -> bool = #name;

        #func
    }
    .into()
}

// Represents the `help = "..."` attribute parameter.
struct HelpAttr(LitStr);

impl Parse for HelpAttr {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        if input.is_empty() {
            return Err(input.error("`help` is a required attribute parameter"));
        }
        let key: Ident = input.parse()?;
        if key != "help" {
            return Err(parse::Error::new_spanned(
                key,
                "`help` is a required attribute parameter",
            ));
        }
        let _: Token![=] = input.parse()?;
        let lit: LitStr = input
            .parse()
            .map_err(|_| input.error("The value of `help` must be a string literal"))?;
        if lit.value().contains('\n') {
            return Err(parse::Error::new_spanned(
                lit,
                "No newlines. Keep it pithy!",
            ));
        }
        Ok(Self(lit))
    }
}

// A parsed shell command function, with its doc comment extracted as the
// long-form description.
struct Command {
    func: ItemFn,
    doc: String,
}

impl Command {
    fn name(&self) -> &Ident {
        &self.func.sig.ident
    }
}

impl Parse for Command {
    fn parse(input: ParseStream) -> parse::Result<Self> {
        let func: ItemFn = input.parse()?;

        // Doc comments are syntactic sugar for #[doc = "..."] attributes,
        // which we can read back as name-value metadata.
        let doc_lines: Vec<String> = func
            .attrs
            .iter()
            .filter_map(|attr| {
                if let Meta::NameValue(nv) = &attr.meta
                    && nv.path.is_ident("doc")
                    && let Expr::Lit(ExprLit {
                        lit: Lit::Str(s), ..
                    }) = &nv.value
                {
                    return Some(s.value());
                }
                None
            })
            .collect();

        let doc = doc_lines.join("\n").to_string();
        if doc.is_empty() {
            return Err(parse::Error::new_spanned(
                &func.sig.ident,
                "Missing a docstring. Shell commands must be documented.",
            ));
        }

        Ok(Self { func, doc })
    }
}
