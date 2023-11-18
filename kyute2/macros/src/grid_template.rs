use crate::CRATE;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    bracketed, parenthesized,
    parse::{Parse, ParseStream},
    parse_macro_input, token, Ident, LitFloat, LitInt, Token, Visibility,
};

enum TrackSize {
    Fixed(f64),
    Flex(f64),
    Auto,
}

impl Parse for TrackSize {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(LitFloat) {
            let literal: LitFloat = input.parse()?;
            let value: f64 = literal.base10_parse()?;
            match literal.suffix() {
                "" | "px" => return Ok(TrackSize::Fixed(value)),
                "fr" => return Ok(TrackSize::Flex(value)),
                _ => {
                    return Err(syn::Error::new(
                        literal.span(),
                        format!("unknown unit: {}", literal.suffix()),
                    ))
                }
            }
        } else if lookahead.peek(LitInt) {
            let literal: LitInt = input.parse()?;
            let value: i32 = literal.base10_parse()?;
            match literal.suffix() {
                "" | "px" => return Ok(TrackSize::Fixed(value as f64)),
                "fr" => return Ok(TrackSize::Flex(value as f64)),
                _ => {
                    return Err(syn::Error::new(
                        literal.span(),
                        format!("unknown unit: {}", literal.suffix()),
                    ))
                }
            }
        } else if lookahead.peek(Ident) {
            let ident: Ident = input.parse()?;
            if ident == "auto" {
                return Ok(TrackSize::Auto);
            }
        }

        Err(syn::Error::new(input.span(), "expected a literal value, or `auto`"))
    }
}

impl TrackSize {
    fn generate(&self) -> TokenStream {
        match self {
            TrackSize::Fixed(value) => {
                quote!(#CRATE::widget::grid::TrackBreadth::Fixed(#value))
            }
            TrackSize::Flex(value) => {
                quote!(#CRATE::widget::grid::TrackBreadth::Flex(#value))
            }
            TrackSize::Auto => {
                quote!(#CRATE::widget::grid::TrackBreadth::Auto)
            }
        }
    }
}

enum TrackListItem {
    Line(Ident),
    Minmax { min: TrackSize, max: TrackSize },
    Size(TrackSize),
}

impl Parse for TrackListItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(LitFloat) || lookahead.peek(LitInt) {
            let size: TrackSize = input.parse()?;
            Ok(TrackListItem::Size(size))
        } else if lookahead.peek(token::Bracket) {
            let content;
            bracketed!(content in input);
            let ident: Ident = content.parse()?;
            if !content.is_empty() {
                return Err(syn::Error::new(content.span(), "extra tokens in line specifier"));
            }
            Ok(TrackListItem::Line(ident))
        } else if lookahead.peek(Ident) {
            let ident: Ident = input.parse()?;
            if ident == "minmax" {
                let content;
                parenthesized!(content in input);
                let min: TrackSize = content.parse()?;
                let _: Token![,] = content.parse()?;
                let max: TrackSize = content.parse()?;
                if !content.is_empty() {
                    return Err(syn::Error::new(content.span(), "extra tokens in minmax"));
                }
                Ok(TrackListItem::Minmax { min, max })
            } else if ident == "auto" {
                Ok(TrackListItem::Size(TrackSize::Auto))
            } else {
                return Err(syn::Error::new(input.span(), "expected `auto` or `minmax()`"));
            }
        } else {
            Err(lookahead.error())
        }
    }
}

impl TrackListItem {
    fn generate(&self) -> Option<TokenStream> {
        match self {
            TrackListItem::Line(_) => None,
            TrackListItem::Minmax { min, max } => {
                let min = min.generate();
                let max = max.generate();
                Some(quote!(
                    #CRATE::widget::grid::TrackSize::minmax(#min,#max)
                ))
            }
            TrackListItem::Size(size) => {
                let size = size.generate();
                Some(quote!(
                    #CRATE::widget::grid::TrackSize::new(#size)
                ))
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

struct GridTemplate {
    vis: Visibility,
    template_name: Ident,
    columns: Vec<TrackListItem>,
    rows: Vec<TrackListItem>,
}

impl Parse for GridTemplate {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vis = input.parse()?;
        let template_name = input.parse()?;
        let _: Token![:] = input.parse()?;

        let mut columns: Vec<TrackListItem> = vec![];
        let mut rows: Vec<TrackListItem> = vec![];

        while !input.peek(Token![/]) && !input.is_empty() {
            columns.push(input.parse()?);
        }

        if input.peek(Token![/]) {
            let _: Token![/] = input.parse()?;
            while !input.is_empty() {
                rows.push(input.parse()?);
            }
        }

        Ok(GridTemplate {
            vis,
            template_name,
            columns,
            rows,
        })
    }
}

impl GridTemplate {
    fn generate(&self) -> TokenStream {
        let vis = &self.vis;
        let name = &self.template_name;
        let column_sizes: Vec<_> = self.columns.iter().filter_map(|item| item.generate()).collect();
        let row_sizes: Vec<_> = self.rows.iter().filter_map(|item| item.generate()).collect();

        let mut lines = TokenStream::new();

        {
            let mut i: u32 = 0;
            for column in self.columns.iter() {
                match column {
                    TrackListItem::Line(ident) => lines.extend(quote!(
                        #vis const #ident: #CRATE::widget::grid::ColumnLineIndex = #CRATE::widget::grid::ColumnLineIndex(#i);
                    )),
                    _ => i += 1
                }
            }
        }
        {
            let mut i: u32 = 0;
            for row in self.rows.iter() {
                match row {
                    TrackListItem::Line(ident) => lines.extend(quote!(
                        #vis const #ident: #CRATE::widget::grid::RowLineIndex = #CRATE::widget::grid::RowLineIndex(#i);
                    )),
                    _ => i += 1,
                }
            }
        }

        quote! {
            #vis const #name: #CRATE::widget::grid::GridTemplate = #CRATE::widget::grid::GridTemplate {
                columns: std::borrow::Cow::Borrowed(&[#( #column_sizes ),*]),
                rows: std::borrow::Cow::Borrowed(&[#( #row_sizes ),*]),
                auto_columns: #CRATE::widget::grid::TrackSize::auto(),
                auto_rows: #CRATE::widget::grid::TrackSize::auto(),
            };

            #lines
        }
    }
}

pub(crate) fn grid_template_impl(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let template = parse_macro_input!(input as GridTemplate);
    template.generate().into()
}
