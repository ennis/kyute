use crate::CRATE;
use proc_macro::{Diagnostic, Level};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::ParseStream,
    punctuated::Punctuated,
    spanned::Spanned,
    visit_mut::{visit_stmt_mut, VisitMut},
    Attribute, FnArg, Local, Pat, Stmt,
};

struct ComposableArgs {
    cached: bool,
}

impl syn::parse::Parse for ComposableArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let idents = Punctuated::<Ident, syn::Token![,]>::parse_terminated(input)?;

        let mut cached = false;
        for ident in idents {
            if ident == "cached" {
                cached = true;
            } else {
                // TODO warn unrecognized attrib
            }
        }
        Ok(ComposableArgs { cached })
    }
}

/// Extract `#[state]` attribute.
fn extract_state_attr(attrs: &mut Vec<Attribute>) -> bool {
    if let Some(pos) = attrs.iter().position(|attr| attr.path.is_ident("state")) {
        if !attrs[pos].tokens.is_empty() {
            Diagnostic::spanned(
                attrs[pos].tokens.span().unwrap(),
                Level::Warning,
                "unknown tokens on `state` attribute",
            )
            .emit();
        }
        // remove the attr
        attrs.remove(pos);
        true
    } else {
        false
    }
}

struct LocalStateCollector {
    visited_first_non_state_stmt: bool,
    locals: Vec<Local>,
}

impl LocalStateCollector {
    fn new() -> LocalStateCollector {
        LocalStateCollector {
            visited_first_non_state_stmt: false,
            locals: vec![],
        }
    }
}

impl VisitMut for LocalStateCollector {
    fn visit_stmt_mut(&mut self, stmt: &mut Stmt) {
        match stmt {
            Stmt::Local(local) => {
                if extract_state_attr(&mut local.attrs) {
                    if self.visited_first_non_state_stmt {
                        Diagnostic::spanned(
                            local.span().unwrap(),
                            Level::Error,
                            "`#[state]` bindings must come before any other statement in a composable function",
                        )
                        .emit();
                    } else {
                        self.locals.push(local.clone());
                    }
                } else {
                    self.visited_first_non_state_stmt = true;
                }
            }
            _ => {
                self.visited_first_non_state_stmt = true;
            }
        }

        visit_stmt_mut(self, stmt);
    }
}

pub fn generate_composable(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // works only on trait declarations
    let mut fn_item: syn::ItemFn = syn::parse_macro_input!(item as syn::ItemFn);
    let attr_args: ComposableArgs = syn::parse_macro_input!(attr as ComposableArgs);

    let vis = &fn_item.vis;
    let attrs = &fn_item.attrs;
    let fn_block = &mut fn_item.block;

    // collect `#[state] let mut state = <initializer>;` statements
    let mut state_collector = LocalStateCollector::new();
    state_collector.visit_block_mut(fn_block);
    // remove state statements from the main block
    let num_state_locals = state_collector.locals.len();
    fn_block.stmts.drain(0..num_state_locals);

    // create prologue statements: load state vars from cache
    let mut prologue = TokenStream::new();
    let mut epilogue = TokenStream::new();

    for (i, local) in state_collector.locals.iter().enumerate() {
        // name of the `cache::Key` variable
        let state_ident = syn::Ident::new(&format!("__state_{}", i), Span::call_site());

        let pat = &local.pat;

        // name of the variable containing the value of the state
        let var_ident = match local.pat {
            Pat::Ident(ref pat_ident) => {
                let ident = &pat_ident.ident;
                quote! { #ident }
            }
            Pat::Type(ref pat_type) => match *pat_type.pat {
                Pat::Ident(ref pat_ident) => {
                    let ident = &pat_ident.ident;
                    quote! { #ident }
                }
                _ => {
                    Diagnostic::spanned(
                        local.pat.span().unwrap(),
                        Level::Error,
                        "unsupported pattern in state binding",
                    )
                    .emit();
                    quote! { () }
                }
            },
            _ => {
                Diagnostic::spanned(
                    local.pat.span().unwrap(),
                    Level::Error,
                    "unsupported pattern in state binding",
                )
                .emit();
                quote! { () }
            }
        };

        // state initializer
        let init = if let Some((_, ref init)) = local.init {
            quote! { #init }
        } else {
            Diagnostic::spanned(
                local.span().unwrap(),
                Level::Error,
                "state binding must have an initializer",
            )
            .emit();
            quote! { () }
        };

        quote! {
            let #state_ident = ::#CRATE::cache::state(|| #init);
            let #pat = #state_ident.get();
        }
        .to_tokens(&mut prologue);

        quote! {
            #state_ident.update(#var_ident);
        }
        .to_tokens(&mut epilogue);
    }

    let altered_fn = if !attr_args.cached {
        let sig = &fn_item.sig;
        //let return_type = &fn_item.sig.output;
        //let debug_name = format!("scope for `{}`", fn_item.sig.ident);
        quote! {
            #[track_caller]
            #(#attrs)* #vis #sig {
                #prologue
                let __result = ::#CRATE::cache::scoped(0, || {
                    #fn_block
                });
                #epilogue
                __result
            }
        }
    } else {
        // convert fn args to tuple
        let args: Vec<_> = fn_item
            .sig
            .inputs
            .iter_mut()
            .filter_map(|arg| match arg {
                FnArg::Receiver(r) => {
                    // FIXME, methods could be cached composables, we just need `self` to be any+clone
                    Diagnostic::spanned(
                        r.span().unwrap(),
                        Level::Error,
                        "methods cannot be cached `composable(cached)` functions: consider using `composable`",
                    )
                    .emit();
                    Some(quote! { self.clone() })
                }
                FnArg::Typed(arg) => {
                    if let Some(pos) = arg.attrs.iter().position(|attr| attr.path.is_ident("uncached")) {
                        // skip uncached argument
                        arg.attrs.remove(pos);
                        return None;
                    }
                    let pat = &arg.pat;
                    let val = quote! {
                        #pat.clone()
                    };
                    Some(val)
                }
            })
            .collect();

        let sig = &fn_item.sig;
        //let return_type = &fn_item.sig.output;
        //let debug_name = format!("memoization wrapper for `{}`", fn_item.sig.ident);

        quote! {
            #[track_caller]
            #(#attrs)* #vis #sig {
                #prologue
                let __result = ::#CRATE::cache::memoize((#(#args,)*), || {
                    #fn_block
                });
                #epilogue
                __result
            }
        }
    };

    //eprintln!("{}", altered_fn);
    altered_fn.into()
}
