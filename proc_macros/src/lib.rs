//! Proc-macros helpers to make the code cleaner.
#![warn(
    missing_docs,
    warnings,
    deprecated_safe,
    future_incompatible,
    keyword_idents,
    let_underscore,
    nonstandard_style,
    refining_impl_trait,
    rust_2018_compatibility,
    rust_2018_idioms,
    rust_2021_compatibility,
    rust_2024_compatibility,
    unused,
    clippy::all,
    clippy::pedantic,
    clippy::style,
    clippy::perf,
    clippy::complexity,
    clippy::correctness,
    clippy::restriction,
    clippy::nursery,
    clippy::cargo
)]
#![allow(
    clippy::single_call_fn,
    clippy::implicit_return,
    clippy::pattern_type_mismatch,
    clippy::blanket_clippy_restriction_lints,
    clippy::missing_trait_methods,
    clippy::question_mark_used,
    clippy::mod_module_files,
    clippy::module_name_repetitions,
    clippy::pub_with_shorthand,
    clippy::unseparated_literal_suffix,
    clippy::else_if_without_else
)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemEnum, LitStr, parse_macro_input, parse_quote};

/// Improve the `this_error` macros to also create documentation from the `error` attributes.
#[proc_macro_attribute]
pub fn doc_error(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(item as ItemEnum);

    for variant in &mut input.variants {
        if let Some(doc) = variant
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("error"))
            .map(|attr| attr.parse_args::<LitStr>().unwrap())
        {
            variant.attrs.push(parse_quote!(#[doc = #doc]));
        }
    }

    let expanded = quote! {
        #[derive(Debug, thiserror::Error)]
        #input
    };
    TokenStream::from(expanded)
}
