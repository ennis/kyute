use proc_macro2::Span;
use proc_macro2::TokenStream;
use quote::ToTokens;
use quote::TokenStreamExt;
use syn::{parse_macro_input, Error};
use syn::parse::{Parse, ParseBuffer, ParseStream, Parser};

struct UpdateGuard {
    pat: syn::Pat,
    tk_eq: syn::Token![=],
    lens: syn::Path,
    src_prop: Option<(syn::Token![in], syn::Ident)>,
}

impl Parse for UpdateGuard {
    fn parse(input: ParseStream) -> syn::Result<Self> {

        let pat = input.parse()?;
        let tk_eq = input.parse()?;
        let lens = input.parse()?;

        let src_prop = if input.peek(syn::Token![in]) {
            Some((input.parse()?, input.parse()?))
        } else {
            None
        };

        Ok(UpdateGuardItem {
            pat,
            tk_eq,
            lens,
            src_prop
        })
    }
}

struct UpdateGuards {
    items: syn::punctuated::Punctuated<UpdateGuard, syn::Token![,]>,
}

impl Parse for Option<UpdateGuards>
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(if input.peek(syn::token::Bracket) {
            let content;
            let bracket = syn::bracketed!(content in input);
            let guards = content.parse()?;
            Some(UpdateGuards {
                items
            })
        } else {
            None
        })
    }
}

enum PropertyBindingSyntax {
    Expr {
        eq: syn::Token![=],
        expr: syn::Expr,
    },
    Shorthand {
        bind: syn::Token![<-],
        lens: syn::Path,
    }
}

impl Parse for PropertyBindingSyntax {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let la = input.lookahead1();
        if la.peek(syn::Token![=]) {
            let eq = input.parse()?;
            let expr = input.parse()?;
            Ok(PropertyBindingSyntax::Expr {
                eq,
                expr
            })
        } else if la.peek(syn::Token![<-]) {
            let bind = input.parse()?;
            let lens = input.parse()?;
            Ok(PropertyBindingSyntax::Shorthand {
                bind,
                lens,
            })
        }
        Err(la.error())
    }
}

struct PropertyBinding {
    guards: Option<UpdateGuards>,
    dot: syn::Token![.],    // not necessary, but looks pretty
    name: syn::Ident,
    rhs: PropertyBindingSyntax,
}

impl Parse for PropertyBinding {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let guards = input.parse()?;
        let dot = input.parse()?;
        let name = input.parse()?;
        let rhs = input.parse()?;

        Ok(PropertyBinding {
            guards,
            dot,
            name,
            rhs,
        })
    }
}

/*struct PropertyBindings {
    bindings: syn::punctuated::Punctuated<PropertyBinding, syn::Token![;]>
}

impl Parse for PropertyBindings {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let bindings = input.parse()?;
        Ok(PropertyBindings {
            bindings
        })
    }
}*/

struct ViewItem {
    guards: Option<UpdateGuards>,
    ty: syn::Type,
    bindings: Option<(syn::token::Paren, syn::punctuated::Punctuated<PropertyBinding, syn::Token![;]>)>,
    contents: Option<(syn::token::Brace, Vec<ViewItem>)>,
}

impl Parse for ViewItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let guards = input.parse()?;
        let ty = input.parse()?;

        let bindings = if input.peek(syn::token::Paren) {
            let content;
            let paren = syn::parenthesized!(content in input);
            Some((paren, content.parse()?))
        } else {
            None
        };

        let contents = if input.peek(syn::token::Brace) {
            let content;
            let brace = syn::braced!(content in input);

            let mut items = Vec::new();
            while !content.is_empty() {
                items.push(content.parse()?);
            }

            Some((brace, items))
        } else {
            None
        };

        Ok(ViewItem {
            guards,
            ty,
            bindings,
            contents
        })
    }
}

struct ViewDefinition {
    props:
}

pub fn derive(input: TokenStream) -> TokenStream {

    // parse leading let bindings
    unimplemented!()
}
