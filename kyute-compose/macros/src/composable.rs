use crate::CRATE;
use proc_macro::{Diagnostic, Level};
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::{
    parse::ParseStream,
    punctuated::Punctuated,
    spanned::Spanned,
    visit_mut::{visit_expr_mut, visit_stmt_mut, VisitMut},
    Attribute, Expr, ExprCall, FnArg, Item, ItemFn, Local, Pat, Stmt,
};

/// Arguments of the `#[composable]` proc-macro.
struct ComposableArgs {
    /// `#[composable(cached)]`
    cached: bool,
    /// `#[composable(live_literals)]`
    live_literals: bool,
}

impl syn::parse::Parse for ComposableArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let idents = Punctuated::<Ident, syn::Token![,]>::parse_terminated(input)?;

        let mut cached = false;
        let mut live_literals = false;
        for ident in idents {
            if ident == "cached" {
                cached = true;
            } else if ident == "live_literals" {
                live_literals = true;
            } else {
                // TODO warn unrecognized attrib
            }
        }
        Ok(ComposableArgs { cached, live_literals })
    }
}

/// Extract `#[state]` attribute.
fn extract_state_attr(attrs: &mut Vec<Attribute>) -> bool {
    if let Some(pos) = attrs.iter().position(|attr| attr.path().is_ident("state")) {
        // TODO
        /*if !attrs[pos].tokens.is_empty() {
            Diagnostic::spanned(
                attrs[pos].tokens.span().unwrap(),
                Level::Warning,
                "unknown tokens on `state` attribute",
            )
            .emit();
        }*/
        // remove the attr
        attrs.remove(pos);
        true
    } else {
        false
    }
}

/// AST visitor that collects `let` bindings annotated with `#[state]`.
///
/// Currently, those bindings must be the first statements of the function body, but this restriction
/// may be removed in the future.
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

/// AST rewriter that wraps string and number literals in a call to the `live_literal` function.
///
/// This rewrites all `ExprLit` nodes in the body of the function, except those in nested items,
/// like nested functions, const item initializers, etc.
///
/// Used by `#[composable(live_literals)]`.
struct LiveLiteralsRewriter;

impl VisitMut for LiveLiteralsRewriter {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Lit(literal) => {
                let literal = literal.clone();
                let span = literal.span().unwrap().source();
                let source_file = span.source_file().path().display().to_string();
                let start_line = span.start().line() as u32;
                let start_column = span.start().column() as u32;
                let end_line = span.end().line() as u32;
                let end_column = span.end().column() as u32;

                let expr_call: ExprCall = syn::parse_quote! {
                    #CRATE::live_literal(#source_file, #start_line, #start_column, #end_line, #end_column, #literal)
                };
                *expr = Expr::Call(expr_call);
            }
            _ => {
                // traverse the rest
                visit_expr_mut(self, expr);
            }
        }
    }

    fn visit_item_mut(&mut self, _item: &mut Item) {
        // skip nested items
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////
// main body
////////////////////////////////////////////////////////////////////////////////////////////////////
pub fn generate_composable(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // works only on trait declarations
    let mut fn_item: ItemFn = syn::parse_macro_input!(item as ItemFn);
    let attr_args: ComposableArgs = syn::parse_macro_input!(attr as ComposableArgs);

    let vis = &fn_item.vis;
    let attrs = &fn_item.attrs;
    let fn_block = &mut fn_item.block;

    // collect `#[state] let mut state = <initializer>;` statements, and
    // remove them from the block
    let mut state_collector = LocalStateCollector::new();
    state_collector.visit_block_mut(fn_block);
    let num_state_locals = state_collector.locals.len();
    // the `#[state]` statements are the first in the main block, the collector checks that.
    fn_block.stmts.drain(0..num_state_locals);

    // if tweakable literals are requested, rewrite the function body
    if attr_args.live_literals {
        LiveLiteralsRewriter.visit_block_mut(fn_block);
    }

    // create prologue statements: load state vars from cache
    let mut prologue = TokenStream::new();
    let mut epilogue = TokenStream::new();

    for (i, local) in state_collector.locals.iter().enumerate() {
        // name of the `cache::Key` variable
        let state_ident = Ident::new(&format!("__state_{}", i), Span::call_site());

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
        let init = if let Some(ref local_init) = local.init {
            let init = &local_init.expr;
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
            let (#state_ident,_) = ::#CRATE::cache_cx::variable(|| #init);
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
                let __result = ::#CRATE::cache_cx::scoped(0, || {
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
                        "methods cannot be `composable(cached)` functions: consider using `composable`",
                    )
                    .emit();
                    Some(quote! { self.clone() })
                }
                FnArg::Typed(arg) => {
                    if let Some(pos) = arg.attrs.iter().position(|attr| attr.path().is_ident("uncached")) {
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
                let __result = ::#CRATE::cache_cx::run_if_changed((#(#args,)*), || {
                    #fn_block
                }).unwrap_or_default();
                #epilogue
                __result
            }
        }
    };

    //eprintln!("{}", altered_fn);
    altered_fn.into()
}
