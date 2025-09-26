use proc_macro::TokenStream;
use quote::quote;
use syn::{Token, Type, parse::Parse, parse_macro_input, punctuated::Punctuated};

struct EmptyDispatchParms {
    target: Type,
    dispatch_types: Punctuated<Type, Token![,]>,
}

impl Parse for EmptyDispatchParms {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let target = input.parse()?;
        input.parse::<Token![=>]>()?;
        let dispatch_types = Punctuated::parse_terminated(input)?;
        Ok(EmptyDispatchParms {
            target,
            dispatch_types,
        })
    }
}

#[proc_macro]
pub fn make_empty_dispatch(tokens: TokenStream) -> TokenStream {
    let EmptyDispatchParms {
        target,
        dispatch_types,
    } = parse_macro_input!(tokens as EmptyDispatchParms);
    let impls = dispatch_types.iter().map(|t| {
        quote! {
            impl Dispatch<#t, ()> for #target {
                fn event(
                    _: &mut Self,
                    _: &#t,
                    _: <#t as Proxy>::Event,
                    _: &(),
                    _: &Connection,
                    _: &QueueHandle<Self>
                ) {}
            }
        }
    });

    quote! {
        #(#impls)*
    }
    .into()
}
