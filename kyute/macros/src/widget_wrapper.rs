use crate::CRATE;
use proc_macro::{Diagnostic, Level};
use quote::quote;
use syn::{spanned::Spanned, Data, Fields};

pub(crate) fn derive_widget_wrapper_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    // filter out enums and unions.
    let input_struct = match input.data {
        Data::Struct(ref s) => s,
        Data::Enum(_) => {
            Diagnostic::spanned(
                input.span().unwrap(),
                Level::Error,
                "`WidgetWrapper` can only be derived on non-unit structs",
            )
            .emit();
            return quote! {}.into();
        }
        Data::Union(_) => {
            Diagnostic::spanned(
                input.span().unwrap(),
                Level::Error,
                "`WidgetWrapper` can only be derived on non-unit structs",
            )
            .emit();
            return quote! {}.into();
        }
    };

    // filter out unit structs
    let input_struct_fields = match input_struct.fields {
        Fields::Named(ref named) => &named.named,
        Fields::Unnamed(ref unnamed) => &unnamed.unnamed,
        Fields::Unit => {
            Diagnostic::spanned(
                input.span().unwrap(),
                Level::Error,
                "`WidgetWrapper` can only be derived on non-unit structs",
            )
            .emit();
            return quote! {}.into();
        }
    };

    // find the field with the #[inner] attribute
    // there should be exactly one
    let inner_fields: Vec<_> = input_struct_fields
        .iter()
        .enumerate()
        .filter(|(_, field)| field.attrs.iter().any(|attr| attr.path.is_ident("inner")))
        .collect();

    let inner_field = if inner_fields.is_empty() {
        // if no fields were annotated with the `#[inner]` attribute, then assume that the first
        // field is the inner widget.
        (0, &input_struct_fields[0])
    } else if inner_fields.len() > 1 {
        let mut diag = Diagnostic::spanned(
            input.ident.span().unwrap(),
            Level::Error,
            "more than one inner widget specified",
        ).note("a struct with `#[derive(WidgetWrapper)]` must have exactly one inner widget to delegate to and more than one was found");

        for f in inner_fields.iter() {
            diag = diag.span_note(f.1.span().unwrap(), "field marked as inner here");
        }
        diag.emit();
        return quote! {}.into();
    } else {
        inner_fields[0]
    };

    let outer_ty = input.ident;
    let access = if let Some(ref ident) = inner_field.1.ident {
        quote! {#ident}
    } else {
        let index = syn::Index::from(inner_field.0);
        quote! {#index}
    };
    let inner_ty = &inner_field.1.ty;

    let (impl_generics, type_generics, where_clause) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics #CRATE::widget::WidgetWrapper for #outer_ty #type_generics #where_clause {
            type Inner = #inner_ty;

            fn inner(&self) -> &Self::Inner {
                &self.#access
            }

            fn inner_mut(&mut self) -> &mut Self::Inner {
                &mut self.#access
            }
        }
    }
    .into()
}
