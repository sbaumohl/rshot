use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemStruct, Token, Type, parse::Parse, parse_macro_input, punctuated::Punctuated};

struct EmptyDispatchParms {
    dispatch_types: Punctuated<Type, Token![,]>,
}

impl Parse for EmptyDispatchParms {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let dispatch_types = Punctuated::parse_terminated(input)?;
        Ok(EmptyDispatchParms { dispatch_types })
    }
}

#[proc_macro_attribute]
pub fn default_dispatch(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemStruct);
    let struct_name = &input.ident;

    let EmptyDispatchParms { dispatch_types } = parse_macro_input!(attr as EmptyDispatchParms);
    let impls = dispatch_types.iter().map(|target| {
        quote! {
            impl Dispatch<#target, ()> for #struct_name {
                fn event(
                    _: &mut Self,
                    _: &#target,
                    _: <#target as Proxy>::Event,
                    _: &(),
                    _: &Connection,
                    _: &QueueHandle<Self>
                ) {}
            }
        }
    });

    quote! {
        #input

        #(#impls)*
    }
    .into()
}
