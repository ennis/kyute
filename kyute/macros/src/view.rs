use crate::CRATE;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::Error;

#[derive(Debug)]
enum Guard {
    Prop {
        tk_colon: syn::Token![:],
        prop: syn::Ident,
    },
    Lens {
        lens: syn::Path,
    },
}

impl Parse for Guard {
    fn parse(input: ParseStream) -> syn::Result<Guard> {
        let content;
        let _bracket = syn::bracketed!(content in input);

        let la = content.lookahead1();

        if la.peek(syn::Token![:]) {
            let tk_colon = content.parse()?;
            let prop = content.parse()?;
            Ok(Guard::Prop { tk_colon, prop })
        } else if la.peek(syn::Ident) {
            let lens = content.parse()?;
            Ok(Guard::Lens { lens })
        } else {
            Err(la.error())
        }
    }
}

#[derive(Debug)]
struct Guards {
    items: Vec<Guard>,
}

impl Parse for Guards {
    fn parse(input: ParseStream) -> syn::Result<Guards> {
        let mut items = Vec::new();
        while input.peek(syn::token::Bracket) {
            items.push(Guard::parse(input)?);
        }
        Ok(Guards { items })
    }
}

#[derive(Debug)]
struct PropertyBinding {
    guards: Guards,
    dot: syn::Token![.],
    name: syn::Ident,
    rhs: PropertyBindingRhs,
    semi: syn::Token![;],
}

#[derive(Debug)]
enum PropertyBindingRhs {
    Expr {
        eq: syn::Token![=],
        expr: syn::Expr,
    },
    Lens {
        bind: syn::Token![<-],
        lens: syn::Path,
    },
}

impl Parse for PropertyBindingRhs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        eprintln!("PropertyBindingRhs ENTER");
        let la = input.lookahead1();
        if la.peek(syn::Token![=]) {
            eprintln!("PropertyBindingRhs::Expr");
            let eq = input.parse()?;
            let expr = input.parse()?;
            Ok(PropertyBindingRhs::Expr { eq, expr })
        } else if la.peek(syn::Token![<-]) {
            eprintln!("PropertyBindingRhs::Lens");
            let bind = input.parse()?;
            let lens = input.parse()?;
            Ok(PropertyBindingRhs::Lens { bind, lens })
        } else {
            eprintln!("PropertyBindingRhs ERROR");
            Err(la.error())
        }
    }
}

impl PropertyBinding {
    fn parse_with_guards(input: ParseStream, guards: Guards) -> syn::Result<PropertyBinding> {
        let dot = input.parse()?;
        let name = input.parse()?;
        let rhs = input.parse()?;
        let semi = input.parse()?;

        Ok(PropertyBinding {
            dot,
            guards,
            name,
            rhs,
            semi,
        })
    }
}

impl Parse for PropertyBinding {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let guards = input.parse()?;
        PropertyBinding::parse_with_guards(input, guards)
    }
}

#[derive(Debug)]
enum ViewContent {
    PropertyBinding(PropertyBinding),
    /// Child view
    Child(ViewItem),
}

impl Parse for ViewContent {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let guards = input.parse()?;
        let la = input.lookahead1();

        if la.peek(syn::Token![.]) {
            let binding = PropertyBinding::parse_with_guards(input, guards)?;
            Ok(ViewContent::PropertyBinding(binding))
        } else if la.peek(syn::Ident) | la.peek(syn::token::Bracket) {
            let child = ViewItem::parse_with_guards(input, guards)?;
            Ok(ViewContent::Child(child))
        } else {
            Err(la.error())
        }
    }
}

#[derive(Debug)]
struct ViewItem {
    guards: Guards,
    ty: syn::Type,
    //bindings: Option<(syn::token::Paren, syn::punctuated::Punctuated<PropertyBinding, syn::Token![,]>)>,
    contents: Option<(syn::token::Brace, Vec<ViewContent>)>,
}

impl ViewItem {
    fn parse_with_guards(input: ParseStream, guards: Guards) -> syn::Result<ViewItem> {
        let ty = input.parse()?;
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
            contents,
        })
    }
}

impl Parse for ViewItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let guards = input.parse()?;
        ViewItem::parse_with_guards(input, guards)
    }
}

#[derive(Debug)]
struct ViewDefinition {
    vis: syn::Visibility,
    name: syn::Ident,
    inputs: Option<(
        syn::token::Paren,
        syn::punctuated::Punctuated<syn::FnArg, syn::Token![,]>,
    )>,
    arrow: syn::Token![->],
    body: ViewItem,
}

impl Parse for ViewDefinition {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vis = input.parse()?;
        let name = input.parse()?;
        let inputs = if input.peek(syn::token::Paren) {
            let content;
            let paren = syn::parenthesized!(content in input);
            Some((
                paren,
                syn::punctuated::Punctuated::parse_terminated(&content)?,
            ))
        } else {
            None
        };

        let arrow = input.parse()?;
        let body = input.parse()?;

        Ok(ViewDefinition {
            vis,
            name,
            inputs,
            arrow,
            body,
        })
    }
}

fn gen_ident(prefix: &str, span: Span) -> syn::Ident {
    //let start = span.start();
    //let end = span.end();

    //syn::Ident::new(&format!("{}_{}_{}_{}_{}", prefix, start.line, start.column, end.line, end.column), Span::call_site())
    syn::Ident::new(prefix, Span::call_site())
}

#[derive(Clone)]
struct GenUpdateFnState<'a> {
    input_prop: &'a syn::Ident,
    input_prop_guarded: bool,
    diff: syn::Ident,
    view: syn::Ident,
}

impl<'a> GenUpdateFnState<'a> {
    fn push_lens<T: Spanned>(&self, syntax: &T) -> GenUpdateFnState {
        GenUpdateFnState {
            diff: gen_ident("diff", syntax.span()),
            ..self.clone()
        }
    }

    fn push_view<T: Spanned>(&self, syntax: &T) -> GenUpdateFnState {
        GenUpdateFnState {
            view: gen_ident("view", syntax.span()),
            ..self.clone()
        }
    }

    fn push_input_prop_guard(&self) -> GenUpdateFnState {
        GenUpdateFnState {
            input_prop_guarded: true,
            ..self.clone()
        }
    }
}

fn gen_property_binding_update(
    binding: &PropertyBinding,
    state: &GenUpdateFnState,
) -> syn::Result<TokenStream> {
    let view = &state.view;
    let prop = &binding.name;

    guarded_update(&binding.guards.items, state, |state| match &binding.rhs {
        PropertyBindingRhs::Lens { lens, .. } => lensed_update(lens, state, |state| {
            let diff = &state.diff;
            let stmt = quote! {
                #view.#prop().update(#diff);
            };
            Ok(stmt)
        }),
        PropertyBindingRhs::Expr { expr, .. } => {
            let stmt = quote! {
                #view.#prop().set(#expr);
            };
            Ok(stmt)
        }
    })
}

fn gen_view_item_update(
    view_item: &ViewItem,
    state: &GenUpdateFnState,
) -> syn::Result<TokenStream> {
    guarded_update(&view_item.guards.items, state, |state| {
        if let Some((_, contents)) = &view_item.contents {
            let mut child_index = 0;
            let mut stmts = Vec::new();
            for c in contents.iter() {
                match c {
                    ViewContent::PropertyBinding(binding) => {
                        let stmt = gen_property_binding_update(binding, state)?;
                        stmts.push(stmt);
                    }
                    ViewContent::Child(child) => {
                        let i = syn::Index::from(child_index);
                        let new_state = state.push_view(&child.ty);
                        let parent_view = &state.view;
                        let child_view = &new_state.view;
                        let body = gen_view_item_update(child, &new_state)?;
                        let block = quote! {
                            {
                                let #child_view = &mut #parent_view.contents_mut().#i;
                                #body
                            }
                        };

                        stmts.push(block);
                        child_index += 1;
                    }
                }
            }

            let body = quote! {
                 #(#stmts)*
            };
            Ok(body)
        } else {
            // No contents
            Ok(TokenStream::new())
        }
    })
}

fn lensed_update(
    lens: &syn::Path,
    state: &GenUpdateFnState,
    body: impl FnOnce(&GenUpdateFnState) -> syn::Result<TokenStream>,
) -> syn::Result<TokenStream> {
    let new_state = state.push_lens(lens);
    let body = body(&new_state)?;

    let root_diff = &state.diff;
    let leaf_diff = &new_state.diff;

    let result = quote! {
        if let Some(#leaf_diff) = #root_diff.focus(#lens) {
            #body
        }
    };

    Ok(result)
}

fn guarded_update(
    guards: &[Guard],
    state: &GenUpdateFnState,
    body: impl FnOnce(&GenUpdateFnState) -> syn::Result<TokenStream>,
) -> syn::Result<TokenStream> {
    if let Some((guard, guards)) = guards.split_first() {
        match guard {
            Guard::Prop { prop, .. } => {
                if state.input_prop_guarded {
                    Err(Error::new(
                        prop.span(),
                        "an input property has already been assigned to this subtree",
                    ))
                } else if prop.to_string() != state.input_prop.to_string() {
                    // not the property we are interested in.
                    Ok(TokenStream::new())
                } else {
                    //
                    let new_state = state.push_input_prop_guard();
                    guarded_update(guards, &new_state, body)
                }
            }
            Guard::Lens { lens } => {
                lensed_update(lens, state, |state| guarded_update(guards, &state, body))
            }
        }
    } else {
        body(state)
    }
}

fn spell_view_type(view: &ViewItem) -> syn::Type {
    // collect content types
    let mut content_types = Vec::new();

    if let Some((_, contents)) = &view.contents {
        for c in contents.iter() {
            let ty = match c {
                ViewContent::Child(view) => spell_view_type(view),
                _ => continue,
            };
            types.push(ty);
        }
    }

    syn::Ty
}

fn generate(view: &ViewDefinition) -> syn::Result<TokenStream> {
    let name = &view.name;
    let vis = &view.vis;
    let root_ty = &view.body.ty;

    let mut update_fns = Vec::new();

    if let Some((_, inputs)) = &view.inputs {
        for (_prop_index, input) in inputs.iter().enumerate() {
            let (ty, prop_name) = match input {
                syn::FnArg::Typed(pat_ty) => match &*pat_ty.pat {
                    syn::Pat::Ident(prop_name) => (&pat_ty.ty, &prop_name.ident),
                    _ => {
                        return Err(syn::Error::new(
                            pat_ty.span(),
                            "unsupported pattern in argument list",
                        ))
                    }
                },
                _ => {
                    return Err(syn::Error::new(
                        view.name.span(),
                        "unexpected receiver in argument list",
                    ));
                }
            };

            let update_fn_name =
                syn::Ident::new(&format!("update_{}", prop_name), Span::call_site());
            let root_view_name = syn::Ident::new("root", Span::call_site());

            let state = GenUpdateFnState {
                input_prop: prop_name,
                input_prop_guarded: false,
                view: root_view_name.clone(),
                diff: prop_name.clone(),
            };

            let body = gen_view_item_update(&view.body, &state)?;

            let f = quote! {
                pub fn #update_fn_name(&mut self, #prop_name: #CRATE::model::Revision<#ty>) {
                    let #root_view_name = &mut self.root;
                    #body
                }
            };
            update_fns.push(f);
        }
    }

    let result = quote! {
        #vis struct #name {
            root: #root_ty,
        }

        impl #name {
            #(#update_fns)*
        }
    };
    Ok(result)
}

pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let view = syn::parse_macro_input!(input as ViewDefinition);

    eprintln!("{:?}", view);

    let result = generate(&view).unwrap_or_else(|err| err.to_compile_error());

    eprintln!("{}", result.to_string());

    result.into()
}
