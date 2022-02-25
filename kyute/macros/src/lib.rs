#![feature(proc_macro_diagnostic)]
extern crate proc_macro;
use proc_macro2::Span;
use quote::{ToTokens, TokenStreamExt};

mod composable;
use composable::generate_composable;

//--------------------------------------------------------------------------------------------------
struct CrateName;
const CRATE: CrateName = CrateName;

impl ToTokens for CrateName {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append(syn::Ident::new("kyute", Span::call_site()))
    }
}

//--------------------------------------------------------------------------------------------------
#[proc_macro_attribute]
pub fn composable(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    generate_composable(attr, item)
}
