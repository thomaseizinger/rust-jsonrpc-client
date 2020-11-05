use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Error, FnArg, ItemStruct, Meta, NestedMeta, Path, TraitItemMethod};
use syn::{ItemTrait, Pat};
use syn::{ReturnType, TraitItem};

#[proc_macro_attribute]
pub fn api(_: TokenStream, input: TokenStream) -> TokenStream {
    match make_new_trait(input) {
        Ok(output) => output,
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_attribute]
pub fn implement(attr: TokenStream, item: TokenStream) -> TokenStream {
    match make_api_impl(item, attr) {
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

        let TraitItemMethod { sig, .. } = method;

        // TODO: Check async and const

        let method_ident = sig.ident;
        let inputs = sig.inputs;

        let arguments = inputs
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
                ()
             },
            ReturnType::Type(_, return_type) => quote! {
                #return_type
            }
        };

        let serialized_arguments = arguments.iter()
            .map(|argument| quote! { ::serde_json::to_value(&#argument)? })
            .collect::<Vec<_>>();


        Ok(quote! {
            fn #method_ident(#inputs) -> Result<#return_type, ::jsonrpc_client::Error<<C as ::jsonrpc_client::SendRequest>::Error>> {
                let parameters = vec![ #(#serialized_arguments),* ];
                let request = ::jsonrpc_client::Request::new_v2(stringify!(#method_ident), parameters);
                let request = ::serde_json::to_string(&request)?;

                let response = self.send_request::<#return_type>(request).map_err(::jsonrpc_client::Error::Client)?;
                let success = Result::from(response.payload).map_err(::jsonrpc_client::Error::JsonRpc)?;

                Ok(success)
            }
        })
    }).collect::<Result<Vec<_>, _>>()?;

    // TODO: handle visibility properly

    Ok(quote! {
        trait #ident<C> where C: ::jsonrpc_client::SendRequest {
            #(#new_methods)*

            fn send_request<P: ::serde::de::DeserializeOwned>(&self, request: String) -> std::result::Result<::jsonrpc_client::Response<P>, <C as ::jsonrpc_client::SendRequest>::Error>;
        }
    }
    .into())
}

fn make_api_impl(item: TokenStream, attr: TokenStream) -> Result<TokenStream, Error> {
    let struct_def = syn::parse::<ItemStruct>(item)?;
    let traits_to_impl = syn::parse::<Path>(attr)?;

    let name = &struct_def.ident;

    if struct_def.fields.is_empty() {
        return Err(Error::new(
            struct_def.span(),
            "struct needs to have at least one field",
        ));
    }

    let (index, client) = if struct_def.fields.len() == 1 {
        struct_def
            .fields
            .iter()
            .enumerate()
            .next()
            .expect("must have field at this stage")
    } else {
        let tagged_inner = struct_def.fields.iter().enumerate().find(|(_, field)| {
            field
                .attrs
                .iter()
                .find(|attr| attr.path.is_ident("jsonrpc_client"))
                .filter(|attr| match attr.parse_meta() {
                    Ok(Meta::List(list)) => match list.nested.first() {
                        Some(NestedMeta::Meta(Meta::Path(path))) => path.is_ident("inner"),
                        _ => false,
                    },
                    _ => false,
                })
                .is_some()
        });

        let named_inner = struct_def.fields.iter().enumerate().find(|(_, field)| {
            field
                .ident
                .as_ref()
                .map(|ident| ident == "inner")
                .unwrap_or(false)
        });

        match tagged_inner.or(named_inner) {
            Some(field) => field,
            None => return Err(Error::new(
                struct_def.span(),
                "struct needs to have either a field named `inner` or one tagged with `#[jsonrpc_client(inner)]`",
            ))
        }
    };

    let inner_type = &client.ty;
    let index = syn::Index::from(index);

    let inner_access = match &client.ident {
        Some(ident) => quote! {
            self.#ident
        },
        None => quote! {
            self.#index
        },
    };

    Ok(quote! {
        #struct_def

        impl #traits_to_impl<#inner_type> for #name {
            fn send_request<P: ::serde::de::DeserializeOwned>(&self, request: String) -> std::result::Result<::jsonrpc_client::Response<P>, <#inner_type as ::jsonrpc_client::SendRequest>::Error> {
                ::jsonrpc_client::SendRequest::send_request(&#inner_access, self.base_url.clone(), request)
            }
        }
    }
        .into())
}
