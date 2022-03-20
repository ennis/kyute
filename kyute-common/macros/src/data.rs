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
//
// Adapted for use in kyute.
use crate::CRATE;
use proc_macro2::{Ident, Literal, Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Data, DataEnum, DataStruct, Error, ExprPath, Meta, NestedMeta};

//show error to tell users of old API that it doesn't work anymore
const BASE_DRUID_DEPRECATED_ATTR_PATH: &str = "druid";
const BASE_DATA_ATTR_PATH: &str = "data";
const IGNORE_ATTR_PATH: &str = "ignore";
const DATA_SAME_FN_ATTR_PATH: &str = "same_fn";

/// The fields for a struct or an enum variant.
#[derive(Debug)]
pub struct Fields<Attrs> {
    pub kind: FieldKind,
    fields: Vec<Field<Attrs>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldKind {
    Named,
    // this also covers Unit; we determine 'unit-ness' based on the number
    // of fields.
    Unnamed,
}

#[derive(Debug)]
pub enum FieldIdent {
    Named(String),
    Unnamed(usize),
}

/*impl FieldIdent {
    pub fn unwrap_named(&self) -> syn::Ident {
        if let FieldIdent::Named(s) = self {
            syn::Ident::new(&s, Span::call_site())
        } else {
            panic!("Unwrap named called on unnamed FieldIdent");
        }
    }
}*/

#[derive(Debug)]
pub struct Field<Attrs> {
    pub ident: FieldIdent,
    pub ty: syn::Type,

    pub attrs: Attrs,
}

#[derive(Debug)]
pub struct DataAttrs {
    /// `true` if this field should be ignored.
    pub ignore: bool,
    pub same_fn: Option<ExprPath>,
}

impl Fields<DataAttrs> {
    pub fn parse_ast(fields: &syn::Fields) -> Result<Self, Error> {
        let kind = match fields {
            syn::Fields::Named(_) => FieldKind::Named,
            syn::Fields::Unnamed(_) | syn::Fields::Unit => FieldKind::Unnamed,
        };

        let fields = fields
            .iter()
            .enumerate()
            .map(|(i, field)| Field::<DataAttrs>::parse_ast(field, i))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Fields { kind, fields })
    }
}

impl<Attrs> Fields<Attrs> {
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Field<Attrs>> {
        self.fields.iter()
    }
}

impl Field<DataAttrs> {
    pub fn parse_ast(field: &syn::Field, index: usize) -> Result<Self, Error> {
        let ident = match field.ident.as_ref() {
            Some(ident) => FieldIdent::Named(ident.to_string().trim_start_matches("r#").to_owned()),
            None => FieldIdent::Unnamed(index),
        };

        let ty = field.ty.clone();

        let mut ignore = false;
        let mut same_fn = None;

        for attr in field.attrs.iter() {
            if attr.path.is_ident(BASE_DRUID_DEPRECATED_ATTR_PATH) {
                panic!(
                    "The 'druid' attribute has been replaced with separate \
                    'lens' and 'data' attributes.",
                );
            } else if attr.path.is_ident(BASE_DATA_ATTR_PATH) {
                match attr.parse_meta()? {
                    Meta::List(meta) => {
                        for nested in meta.nested.iter() {
                            match nested {
                                NestedMeta::Meta(Meta::Path(path)) if path.is_ident(IGNORE_ATTR_PATH) => {
                                    if ignore {
                                        return Err(Error::new(nested.span(), "Duplicate attribute"));
                                    }
                                    ignore = true;
                                }
                                NestedMeta::Meta(Meta::NameValue(meta))
                                    if meta.path.is_ident(DATA_SAME_FN_ATTR_PATH) =>
                                {
                                    if same_fn.is_some() {
                                        return Err(Error::new(meta.span(), "Duplicate attribute"));
                                    }

                                    let path = parse_lit_into_expr_path(&meta.lit)?;
                                    same_fn = Some(path);
                                }
                                other => return Err(Error::new(other.span(), "Unknown attribute")),
                            }
                        }
                    }
                    other => {
                        return Err(Error::new(
                            other.span(),
                            "Expected attribute list (the form #[data(one, two)])",
                        ));
                    }
                }
            }
        }
        Ok(Field {
            ident,
            ty,
            attrs: DataAttrs { ignore, same_fn },
        })
    }

    /// The tokens to be used as the function for 'same'.
    pub fn same_fn_path_tokens(&self) -> TokenStream {
        match self.attrs.same_fn {
            Some(ref f) => quote!(#f),
            None => {
                let span = Span::call_site();
                quote_spanned!(span=> #CRATE::Data::same)
            }
        }
    }
}

impl<Attrs> Field<Attrs> {
    pub fn ident_tokens(&self) -> TokenTree {
        match self.ident {
            FieldIdent::Named(ref s) => Ident::new(s, Span::call_site()).into(),
            FieldIdent::Unnamed(num) => Literal::usize_unsuffixed(num).into(),
        }
    }

    pub fn ident_string(&self) -> String {
        match self.ident {
            FieldIdent::Named(ref s) => s.clone(),
            FieldIdent::Unnamed(num) => num.to_string(),
        }
    }
}

fn parse_lit_into_expr_path(lit: &syn::Lit) -> Result<ExprPath, Error> {
    let string = if let syn::Lit::Str(lit) = lit {
        lit
    } else {
        return Err(Error::new(lit.span(), "expected str, found... something else"));
    };

    let tokens = syn::parse_str(&string.value())?;
    syn::parse2(tokens)
}

/*fn parse_lit_into_ident(lit: &syn::Lit) -> Result<Ident, Error> {
    let ident = if let syn::Lit::Str(lit) = lit {
        Ident::new(&lit.value(), lit.span())
    } else {
        return Err(Error::new(
            lit.span(),
            "expected str, found... something else",
        ));
    };

    Ok(ident)
}*/

pub(crate) fn derive_data_impl(input: syn::DeriveInput) -> Result<proc_macro2::TokenStream, syn::Error> {
    match &input.data {
        Data::Struct(s) => derive_struct(&input, s),
        Data::Enum(e) => derive_enum(&input, e),
        Data::Union(u) => Err(syn::Error::new(
            u.union_token.span(),
            "Data implementations cannot be derived from unions",
        )),
    }
}

fn derive_struct(input: &syn::DeriveInput, s: &DataStruct) -> Result<proc_macro2::TokenStream, syn::Error> {
    let ident = &input.ident;
    let impl_generics = generics_bounds(&input.generics);
    let (_, ty_generics, where_clause) = &input.generics.split_for_impl();

    let fields = Fields::<DataAttrs>::parse_ast(&s.fields)?;

    let diff = if fields.len() > 0 {
        let same_fns = fields
            .iter()
            .filter(|f| !f.attrs.ignore)
            .map(Field::same_fn_path_tokens);
        let fields = fields.iter().filter(|f| !f.attrs.ignore).map(Field::ident_tokens);
        quote!( #( #same_fns(&self.#fields, &other.#fields) )&&* )
    } else {
        quote!(true)
    };

    let res = quote! {
        impl<#impl_generics> ::#CRATE::Data for #ident #ty_generics #where_clause {
            fn same(&self, other: &Self) -> bool {
                #diff
            }
        }
    };

    Ok(res)
}

fn ident_from_str(s: &str) -> proc_macro2::Ident {
    proc_macro2::Ident::new(s, proc_macro2::Span::call_site())
}

fn is_c_style_enum(s: &DataEnum) -> bool {
    s.variants.iter().all(|variant| match &variant.fields {
        syn::Fields::Named(fs) => fs.named.is_empty(),
        syn::Fields::Unnamed(fs) => fs.unnamed.is_empty(),
        syn::Fields::Unit => true,
    })
}

fn derive_enum(input: &syn::DeriveInput, s: &DataEnum) -> Result<proc_macro2::TokenStream, syn::Error> {
    let ident = &input.ident;
    let impl_generics = generics_bounds(&input.generics);
    let (_, ty_generics, where_clause) = &input.generics.split_for_impl();

    if is_c_style_enum(s) {
        let res = quote! {
            impl<#impl_generics> ::#CRATE::Data for #ident #ty_generics #where_clause {
                fn same(&self, other: &Self) -> bool { self == other }
            }
        };
        return Ok(res);
    }

    let cases: Vec<proc_macro2::TokenStream> = s
        .variants
        .iter()
        .map(|variant| {
            let fields = Fields::<DataAttrs>::parse_ast(&variant.fields)?;
            let variant = &variant.ident;

            // the various inner `same()` calls, to the right of the match arm.
            let tests: Vec<_> = fields
                .iter()
                .filter(|field| !field.attrs.ignore)
                .map(|field| {
                    let same_fn = field.same_fn_path_tokens();
                    let var_left = ident_from_str(&format!("__self_{}", field.ident_string()));
                    let var_right = ident_from_str(&format!("__other_{}", field.ident_string()));
                    quote!( #same_fn(#var_left, #var_right) )
                })
                .collect();

            if let FieldKind::Named = fields.kind {
                let lefts: Vec<_> = fields
                    .iter()
                    .map(|field| {
                        let ident = field.ident_tokens();
                        let var = ident_from_str(&format!("__self_{}", field.ident_string()));
                        quote!( #ident: #var )
                    })
                    .collect();
                let rights: Vec<_> = fields
                    .iter()
                    .map(|field| {
                        let ident = field.ident_tokens();
                        let var = ident_from_str(&format!("__other_{}", field.ident_string()));
                        quote!( #ident: #var )
                    })
                    .collect();

                Ok(quote! {
                    (#ident :: #variant { #( #lefts ),* }, #ident :: #variant { #( #rights ),* }) => {
                        #( #tests )&&*
                    }
                })
            } else {
                let vars_left: Vec<_> = fields
                    .iter()
                    .map(|field| ident_from_str(&format!("__self_{}", field.ident_string())))
                    .collect();
                let vars_right: Vec<_> = fields
                    .iter()
                    .map(|field| ident_from_str(&format!("__other_{}", field.ident_string())))
                    .collect();

                if fields.iter().count() > 0 {
                    Ok(quote! {
                        ( #ident :: #variant( #(#vars_left),* ),  #ident :: #variant( #(#vars_right),* )) => {
                            #( #tests )&&*
                        }
                    })
                } else {
                    Ok(quote! {
                        ( #ident :: #variant ,  #ident :: #variant ) => { true }
                    })
                }
            }
        })
        .collect::<Result<Vec<proc_macro2::TokenStream>, syn::Error>>()?;

    let res = quote! {
        impl<#impl_generics> ::#CRATE::Data for #ident #ty_generics #where_clause {
            fn same(&self, other: &Self) -> bool {
                match (self, other) {
                    #( #cases ),*
                    _ => false,
                }
            }
        }
    };

    Ok(res)
}

fn generics_bounds(generics: &syn::Generics) -> proc_macro2::TokenStream {
    let res = generics.params.iter().map(|gp| {
        use syn::GenericParam::*;
        match gp {
            Type(ty) => {
                let ident = &ty.ident;
                let bounds = &ty.bounds;
                if bounds.is_empty() {
                    quote_spanned!(ty.span()=> #ident : ::#CRATE::Data)
                } else {
                    quote_spanned!(ty.span()=> #ident : #bounds + ::#CRATE::Data)
                }
            }
            Lifetime(lf) => quote!(#lf),
            Const(cst) => quote!(#cst),
        }
    });

    quote!( #( #res, )* )
}
