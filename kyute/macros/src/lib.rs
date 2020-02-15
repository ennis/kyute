//! Implementation of the `view!` proc-macro
#![recursion_limit = "256"]
#![feature(proc_macro_diagnostic)]
extern crate proc_macro;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::ToTokens;
use quote::TokenStreamExt;

mod data;
mod view;

//--------------------------------------------------------------------------------------------------
struct CrateName;
const CRATE: CrateName = CrateName;

impl ToTokens for CrateName {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append(syn::Ident::new("kyute", Span::call_site()))
    }
}

//--------------------------------------------------------------------------------------------------
#[proc_macro]
pub fn view(input: TokenStream) -> TokenStream {
    view::derive(input)
}

//--------------------------------------------------------------------------------------------------
#[proc_macro_derive(Data, attributes(argument))]
pub fn data_derive(input: TokenStream) -> TokenStream {
    data::derive(input)
}
