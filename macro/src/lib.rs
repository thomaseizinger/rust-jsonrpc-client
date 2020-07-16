use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Error, FnArg, TraitItemMethod};
use syn::{ItemTrait, Pat};
use syn::{ReturnType, TraitItem};

#[proc_macro_attribute]
pub fn api(_: TokenStream, input: TokenStream) -> TokenStream {
    match make_new_trait(input) {
        Ok(output) => output,
        Err(e) => e.to_compile_error().into(),
    }
}

fn make_new_trait(input: TokenStream) -> Result<TokenStream, Error> {
    let trait_def = syn::parse::<ItemTrait>(input)?;

    let ItemTrait { items, ident, .. } = trait_def;

    let methods: Vec<TraitItemMethod> = items
        .into_iter()
        .map(|item| match item {
            TraitItem::Method(method) => Ok(method),
            _ => Err(Error::new(item.span(), "trait must only define methods")),
        })
        .collect::<Result<Vec<_>, _>>()?;

    let new_methods = methods.into_iter().map(|method| {
        if method.default.is_some() {
            return Err(Error::new(
                method.span(),
                "trait method must not have a default implementation already",
            ));
        }

        let TraitItemMethod { sig, attrs, .. } = method;

        // TODO: Check async and const

        let method_ident = sig.ident;
        let inputs = sig.inputs;

        let _arguments = inputs
            .iter()
            .filter_map(|input| match input {
                FnArg::Receiver(_) => None,
                FnArg::Typed(arg) => match &*arg.pat {
                    Pat::Ident(ident) => Some(ident.ident.clone()),
                    _ => None,
                },
            })
            .collect::<Vec<_>>();

        // TODO: Assert that all arguments implement Serialize

        let return_type = match sig.output {
            ReturnType::Default => quote! {
                Result<(), ::jsonrpc_client::Error<<Self as ::jsonrpc_client::SendRequest>::Error>>
             },
            ReturnType::Type(_, return_type) => quote! {
                Result<#return_type, ::jsonrpc_client::Error<<Self as ::jsonrpc_client::SendRequest>::Error>>
            }
        };

        Ok(quote! {
            fn #method_ident(#inputs) -> #return_type {
                unimplemented!()
            }
        })
    }).collect::<Result<Vec<_>, _>>()?;

    // TODO: handle visibility properly

    Ok(quote! {
        trait #ident where Self: ::jsonrpc_client::SendRequest {
            #(#new_methods)*
        }
    }
    .into())
}
