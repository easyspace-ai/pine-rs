//! Proc-macros for `pine-stdlib` builtin registration.
//!
//! Use `#[pine_builtin(...)]` on a function with signature
//! `fn name(args: &[pine_runtime::value::Value]) -> pine_runtime::value::Value`.
//! The macro emits `pub(crate) fn register_<api_name>(registry: &mut FunctionRegistry)`.

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{parse_macro_input, Ident, ItemFn, LitBool, LitInt, LitStr, Meta, Result, Token};

struct BuiltinAttrs {
    name: String,
    namespace: Option<String>,
    required_args: usize,
    optional_args: usize,
    variadic: bool,
    returns_series: bool,
    hot: bool,
}

impl Parse for BuiltinAttrs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut name: Option<String> = None;
        let mut namespace: Option<String> = None;
        let mut required_args: usize = 0;
        let mut optional_args: usize = 0;
        let mut variadic = false;
        let mut returns_series = false;
        let mut hot = false;

        let params = Punctuated::<Meta, Token![,]>::parse_terminated(input)?;
        for meta in params {
            match meta {
                Meta::Path(p) if p.is_ident("variadic") => {
                    variadic = true;
                }
                Meta::Path(p) if p.is_ident("returns_series") => {
                    returns_series = true;
                }
                Meta::Path(p) if p.is_ident("hot") => {
                    hot = true;
                }
                Meta::NameValue(nv) if nv.path.is_ident("name") => {
                    let v: LitStr = syn::parse2(nv.value.to_token_stream())?;
                    name = Some(v.value());
                }
                Meta::NameValue(nv) if nv.path.is_ident("namespace") => {
                    let v: LitStr = syn::parse2(nv.value.to_token_stream())?;
                    namespace = Some(v.value());
                }
                Meta::NameValue(nv) if nv.path.is_ident("required_args") => {
                    let v: LitInt = syn::parse2(nv.value.to_token_stream())?;
                    required_args = v.base10_parse()?;
                }
                Meta::NameValue(nv) if nv.path.is_ident("optional_args") => {
                    let v: LitInt = syn::parse2(nv.value.to_token_stream())?;
                    optional_args = v.base10_parse()?;
                }
                Meta::NameValue(nv) if nv.path.is_ident("variadic") => {
                    let v: LitBool = syn::parse2(nv.value.to_token_stream())?;
                    variadic = v.value();
                }
                Meta::NameValue(nv) if nv.path.is_ident("returns_series") => {
                    let v: LitBool = syn::parse2(nv.value.to_token_stream())?;
                    returns_series = v.value();
                }
                Meta::NameValue(nv) if nv.path.is_ident("hot") => {
                    let v: LitBool = syn::parse2(nv.value.to_token_stream())?;
                    hot = v.value();
                }
                other => {
                    return Err(syn::Error::new(
                        other.span(),
                        "expected name = \"...\", namespace = \"...\", required_args, optional_args, variadic, returns_series, hot",
                    ));
                }
            }
        }

        let Some(name) = name else {
            return Err(input.error("`name = \"...\"` is required"));
        };

        Ok(BuiltinAttrs {
            name,
            namespace,
            required_args,
            optional_args,
            variadic,
            returns_series,
            hot,
        })
    }
}

/// Registers the attributed function with `FunctionRegistry`.
#[proc_macro_attribute]
pub fn pine_builtin(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attrs = parse_macro_input!(attr as BuiltinAttrs);
    let item_fn = parse_macro_input!(item as ItemFn);

    let impl_ident = &item_fn.sig.ident;
    let api_name = attrs.name.replace('.', "_");
    let register_ident = Ident::new(
        &format!("register_{api_name}"),
        proc_macro2::Span::call_site(),
    );

    let lit_name = LitStr::new(&attrs.name, proc_macro2::Span::call_site());
    let req = attrs.required_args;
    let opt = attrs.optional_args;

    let ns = attrs
        .namespace
        .as_ref()
        .map(|n| {
            let lit = LitStr::new(n, proc_macro2::Span::call_site());
            quote! { .with_namespace(#lit) }
        })
        .unwrap_or_default();

    let variadic = if attrs.variadic {
        quote! { .with_variadic() }
    } else {
        quote! {}
    };

    let series = if attrs.returns_series {
        quote! { .with_series_return() }
    } else {
        quote! {}
    };

    let reg_call = if attrs.hot {
        quote! { registry.register_hot(meta, func); }
    } else {
        quote! { registry.register(meta, func); }
    };

    let expanded = quote! {
        #item_fn

        pub(crate) fn #register_ident(registry: &mut crate::registry::FunctionRegistry) {
            let meta = crate::registry::FunctionMeta::new(#lit_name)
                .with_required_args(#req)
                .with_optional_args(#opt)
                #ns
                #variadic
                #series;
            let func: crate::registry::BuiltinFn = ::std::sync::Arc::new(#impl_ident);
            #reg_call
        }
    };

    expanded.into()
}
