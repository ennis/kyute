use crate::CRATE;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};
use syn::{
    parse::{ParseStream, Parser},
    punctuated::Punctuated,
    spanned::Spanned,
    FnArg, Path,
};

struct ComposableArgs {
    uncached: bool,
}

impl syn::parse::Parse for ComposableArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let idents = Punctuated::<Ident, syn::Token![,]>::parse_terminated(input)?;

        let mut uncached = false;
        for ident in idents {
            if ident == "uncached" {
                uncached = true;
            } else {
                // TODO warn unrecognized attrib
            }
        }
        Ok(ComposableArgs { uncached })
    }
}

pub fn generate_composable(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    // works only on trait declarations
    let mut fn_item: syn::ItemFn = syn::parse_macro_input!(item as syn::ItemFn);
    let attr_args: ComposableArgs = syn::parse_macro_input!(attr as ComposableArgs);

    let vis = &fn_item.vis;
    let attrs = &fn_item.attrs;
    let orig_block = &fn_item.block;

    let altered_fn = if attr_args.uncached {
        let sig = &fn_item.sig;
        let return_type = &fn_item.sig.output;
        //let debug_name = format!("scope for `{}`", fn_item.sig.ident);
        quote! {
            #[track_caller]
            #(#attrs)* #vis #sig {
                ::#CRATE::cache::scoped(0, move || {
                    #orig_block
                })
            }
        }
    } else {
        // convert fn args to tuple
        let args: Vec<_> = fn_item.sig
            .inputs
            .iter_mut()
            .filter_map(|arg| match arg {
                FnArg::Receiver(r) => {
                    // FIXME, methods could be cached composables, we just need `self` to be any+clone
                    Some(syn::Error::new(r.span(), "methods cannot be cached `composable` functions: consider using `composable(uncached)`")
                        .to_compile_error())
                }
                FnArg::Typed(arg) => {
                    if let Some(pos) = arg.attrs.iter().position(|attr| attr.path.is_ident("uncached")) {
                        // skip uncached argument
                        arg.attrs.remove(pos);
                        return None
                    }
                    let pat = &arg.pat;
                    let val = quote! {
                        #pat.clone()
                    };
                    Some(val)
                },
            })
            .collect();

        let sig = &fn_item.sig;
        let return_type = &fn_item.sig.output;
        //let debug_name = format!("memoization wrapper for `{}`", fn_item.sig.ident);

        quote! {
            #[track_caller]
            #(#attrs)* #vis #sig {
                ::#CRATE::cache::memoize((#(#args,)*), move || {
                    #orig_block
                })
            }
        }
    };

    eprintln!("{}", altered_fn);
    altered_fn.into()
}
