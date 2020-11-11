use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    spanned::Spanned, Error, Field, Fields, FnArg, ItemStruct, ItemTrait, Lit, Meta, MetaNameValue,
    NestedMeta, Pat, Path, ReturnType, TraitItem, TraitItemMethod,
};

#[proc_macro_attribute]
pub fn api(attr: TokenStream, item: TokenStream) -> TokenStream {
    match make_new_trait(item, attr) {
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

enum Version {
    One,
    Two,
}

fn make_new_trait(input: TokenStream, attr: TokenStream) -> Result<TokenStream, Error> {
    let trait_def = syn::parse::<ItemTrait>(input)?;
    let version = if attr.is_empty() {
        Version::Two
    } else {
        let meta_name_value = syn::parse::<MetaNameValue>(attr)?;

        if !meta_name_value.path.is_ident("version") {
            return Err(Error::new(
                meta_name_value.path.span(),
                "invalid configuration attribute, currently only `version` is supported",
            ));
        }

        match meta_name_value.lit {
            Lit::Str(str_lit) if str_lit.value() == "1.0" => Version::One,
            Lit::Str(str_lit) if str_lit.value() == "2.0" => Version::Two,
            _ => {
                return Err(Error::new(
                    meta_name_value.lit.span(),
                    r#"argument to `version` must be either "1.0" or "2.0""#,
                ))
            }
        }
    };

    let methods: Vec<TraitItemMethod> = trait_def
        .items
        .into_iter()
        .map(|item| match item {
            TraitItem::Method(method) => Ok(method),
            _ => Err(Error::new(item.span(), "trait must only define methods")),
        })
        .collect::<Result<Vec<_>, _>>()?;

    let new_methods = methods.iter().map(|method| {
        if method.default.is_some() {
            return Err(Error::new(
                method.default.span(),
                "trait method must not have a default implementation",
            ));
        }

        if method.sig.asyncness.is_none() {
            return Err(Error::new(
                method.sig.span(),
                "trait methods have to be async",
            ));
        }

        let arguments = method.sig.inputs
            .iter()
            .filter_map(|input| match input {
                FnArg::Receiver(_) => None,
                FnArg::Typed(arg) => match &*arg.pat {
                    Pat::Ident(ident) => Some((&ident.ident, &arg.ty)),
                    _ => None,
                },
            })
            .collect::<Vec<_>>();

        let return_type = match &method.sig.output {
            ReturnType::Default => quote! {
               ()
            },
            ReturnType::Type(_, return_type) => quote! {
                #return_type
            },
        };

        let serialized_arguments = arguments
            .iter()
            .map(|(argument, ty)| quote_spanned! { ty.span() => ::serde_json::to_value(&#argument)? })
            .collect::<Vec<_>>();

        let new_request_fn = match version {
            Version::One => quote! { new_v1 },
            Version::Two => quote! { new_v2 },
        };
        let method_ident = &method.sig.ident;
        let inputs = &method.sig.inputs;

        let send_request_call = match &method.sig.output {
            ReturnType::Default => quote! {
               self.send_request::<#return_type>(request).await.map_err(::jsonrpc_client::Error::Client)?;
            },
            ReturnType::Type(_, return_type) => quote_spanned! { return_type.span() =>
                self.send_request::<#return_type>(request).await.map_err(::jsonrpc_client::Error::Client)?;
            },
        };

        Ok(quote! {
            async fn #method_ident(#inputs) -> Result<#return_type, ::jsonrpc_client::Error<<C as ::jsonrpc_client::SendRequest>::Error>> {
                let parameters = vec![ #(#serialized_arguments),* ];
                let request = ::jsonrpc_client::Request::#new_request_fn(stringify!(#method_ident), parameters);
                let request = ::serde_json::to_string(&request)?;

                let response = #send_request_call
                let success = Result::from(response.payload).map_err(::jsonrpc_client::Error::JsonRpc)?;

                Ok(success)
            }
        })
    }).collect::<Result<Vec<_>, _>>()?;

    let trait_ident = trait_def.ident;
    let vis = trait_def.vis;

    Ok(quote! {
        #[async_trait::async_trait]
        #vis trait #trait_ident<C> where C: ::jsonrpc_client::SendRequest {
            #(#new_methods)*

            async fn send_request<P: ::serde::de::DeserializeOwned>(&self, request: String) -> std::result::Result<::jsonrpc_client::Response<P>, <C as ::jsonrpc_client::SendRequest>::Error>;
        }
    }.into())
}

fn make_api_impl(item: TokenStream, attr: TokenStream) -> Result<TokenStream, Error> {
    let mut struct_def = syn::parse::<ItemStruct>(item)?;
    let traits_to_impl = syn::parse::<Path>(attr)?;

    let name = &struct_def.ident;

    if struct_def.fields.is_empty() {
        return Err(Error::new(
            struct_def.span(),
            "struct needs to have a client and a base URL",
        ));
    }

    let (client_access, client_ty) = {
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
            Some((_, Field { ident: Some(ident), ty, .. })) => (quote! { self.#ident }, ty),
            Some((index, Field { ident: None, ty, .. })) => {
                let index = syn::Index::from(index);

                (
                    quote! { self.#index },
                    ty
                )
            },
            None => return Err(Error::new(
                struct_def.fields.span(),
                "struct needs to have either a field named `inner` or one tagged with `#[jsonrpc_client(inner)]`",
            ))
        }
    };

    let base_url_access = {
        let tagged_inner = struct_def.fields.iter().enumerate().find(|(_, field)| {
            field
                .attrs
                .iter()
                .find(|attr| attr.path.is_ident("jsonrpc_client"))
                .filter(|attr| match attr.parse_meta() {
                    Ok(Meta::List(list)) => match list.nested.first() {
                        Some(NestedMeta::Meta(Meta::Path(path))) => path.is_ident("base_url"),
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
                .map(|ident| ident == "base_url")
                .unwrap_or(false)
        });

        match tagged_inner.or(named_inner) {
            Some((_, Field { ident: Some(ident),  .. })) => quote! { self.#ident },
            Some((index, Field { ident: None,  .. })) => {
                let index = syn::Index::from(index);

                quote! { self.#index }
            },
            None => return Err(Error::new(
                struct_def.fields.span(),
                "struct needs to have either a field named `base_url` or one tagged with `#[jsonrpc_client(base_url)]`",
            ))
        }
    };

    let trait_impl = quote! {
        #[async_trait::async_trait]
        impl #traits_to_impl<#client_ty> for #name {
            async fn send_request<P: ::serde::de::DeserializeOwned>(&self, request: String) -> std::result::Result<::jsonrpc_client::Response<P>, <#client_ty as ::jsonrpc_client::SendRequest>::Error> {
                ::jsonrpc_client::SendRequest::send_request(&#client_access, #base_url_access.clone(), request).await
            }
        }
    };

    // remove all `jsonrpc_client` attributes from the struct definition
    let fields = match &mut struct_def.fields {
        Fields::Named(named_fields) => named_fields.named.iter_mut(),
        Fields::Unnamed(unnamed_fields) => unnamed_fields.unnamed.iter_mut(),
        Fields::Unit => unreachable!("struct must not be a unit struct"),
    };
    for field in fields {
        field
            .attrs
            .retain(|attr| !attr.path.is_ident("jsonrpc_client"));
    }

    Ok(quote! {
        #struct_def

        #trait_impl
    }
    .into())
}
