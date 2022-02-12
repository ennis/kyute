// This file contains altered parts of druid.
//
// Copyright 2019 The Druid Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![feature(proc_macro_diagnostic)]
extern crate proc_macro;
use proc_macro2::Span;
use quote::{ToTokens, TokenStreamExt};

mod composable;
mod data;

use composable::generate_composable;
use data::derive_data_impl;
//use resource::generate_resource_directory;

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
pub fn composable(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    generate_composable(attr, item)
}

// Originally part of druid.
/// Generates implementations of the `Data` trait.
///
/// This macro supports a `data` field attribute with the following arguments:
///
/// - `#[data(ignore)]` makes the generated `Data::same` function skip comparing this field.
/// - `#[data(same_fn="foo")]` uses the function `foo` for comparing this field. `foo` should
///    be the name of a function with signature `fn(&T, &T) -> bool`, where `T` is the type of
///    the field.
///
/// # Example
///
/// ```rust
/// use druid_derive::Data;
///
/// #[derive(Clone, Data)]
/// struct State {
///     number: f64,
///     // `Vec` doesn't implement `Data`, so we need to either ignore it or supply a `same_fn`.
///     #[data(same_fn="PartialEq::eq")]
///     indices: Vec<usize>,
///     // This is just some sort of cache; it isn't important for sameness comparison.
///     #[data(ignore)]
///     cached_indices: Vec<usize>,
/// }
/// ```
#[proc_macro_derive(Data, attributes(data))]
pub fn derive_data(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    derive_data_impl(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
