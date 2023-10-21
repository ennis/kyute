#![feature(proc_macro_diagnostic)]
#![feature(proc_macro_span)]
extern crate proc_macro;
use proc_macro2::Span;
use quote::{ToTokens, TokenStreamExt};

mod grid_template;
use grid_template::grid_template_impl;

//--------------------------------------------------------------------------------------------------
struct CrateName;
const CRATE: CrateName = CrateName;

impl ToTokens for CrateName {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append(syn::Ident::new("kyute2", Span::call_site()))
    }
}

//--------------------------------------------------------------------------------------------------

/// # Examples
///```ignore
/// grid_template! { GRID:[START] 100px 1fr 1fr [END] / [TOP] auto [BOTTOM] }
///```
#[proc_macro]
pub fn grid_template(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    grid_template_impl(tokens)
}
