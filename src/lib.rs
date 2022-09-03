use proc_macro::TokenStream;
use syn::{Field, Fields, FieldsNamed, FieldsUnnamed, ItemEnum, parse_macro_input, Pat, ReturnType, Token, TraitItem, TraitItemMethod, Type, Variant, Visibility};
use quote::{format_ident, quote};
use syn::ItemTrait;
use syn::{Ident};
use syn::punctuated::Punctuated;
use convert_case::{Case, Casing};
use syn::FnArg::{Typed};
use syn::spanned::Spanned;
use syn::token::Brace;

fn snake_to_pascal(ident: &Ident) -> Ident {
    let pascal = ident.to_string().to_case(Case::Pascal);
    Ident::new(&pascal, ident.span())
}

fn request_variants(methods: &Vec<&TraitItemMethod>) -> Punctuated<Variant, Token![,]> {
    let mut variants = Punctuated::new();

    for &method in methods {
        let mut fields_named = FieldsNamed {
            brace_token: Brace { span: method.span() },
            named: Default::default(),
        };

        for arg in &method.sig.inputs {
            match arg {
                Typed(a) => {
                    match &*a.pat {
                        Pat::Ident(ident) => {
                            fields_named.named.push(
                                Field {
                                    attrs: vec![],
                                    vis: Visibility::Inherited,
                                    ident: Some(ident.ident.clone()),
                                    colon_token: None,
                                    ty: *a.ty.clone(),
                                })
                        }
                        // #[cfg_attr(test, deny(non_exhaustive_omitted_patterns))]
                        _ => { panic!("Only 'simple' arguments are supported") }
                    }
                }
                _ => ()
            }
        }

        variants.push(Variant {
            attrs: vec![],
            ident: snake_to_pascal(&method.sig.ident),
            fields: Fields::Named(fields_named),
            discriminant: None,
        })
    }
    variants
}

fn response_variants(methods: &Vec<&TraitItemMethod>) -> Punctuated<Variant, Token![,]> {
    let mut variants = Punctuated::new();
    for &method in methods {
        let mut fields: Punctuated<Field, Token![,]> = Punctuated::new();
        match &method.sig.output {
            ReturnType::Default => {}
            ReturnType::Type(_, b) => {
                fields.push(Field {
                    attrs: vec![],
                    vis: Visibility::Inherited,
                    ident: None,
                    colon_token: None,
                    ty: *b.clone(),
                });
            }
        }

        variants.push(Variant {
            attrs: vec![],
            ident: snake_to_pascal(&method.sig.ident),
            fields: Fields::Unnamed(
                FieldsUnnamed {
                    paren_token: Default::default(),
                    unnamed: fields,
                }),
            discriminant: None,
        })
    }
    variants
}

fn enums(item: &ItemTrait) -> (syn::ItemEnum, syn::ItemEnum) {
    let methods: Vec<&TraitItemMethod> = item.items.iter().filter_map(|i| match i {
        TraitItem::Method(method) => Some(method),
        _ => None
    }).collect();

    (
        ItemEnum {
            attrs: vec![],
            vis: Visibility::Inherited,
            enum_token: Default::default(),
            ident: format_ident!("{}Request", item.ident),
            generics: Default::default(),
            brace_token: Default::default(),
            variants: request_variants(&methods),
        },
        ItemEnum {
            attrs: vec![],
            vis: Visibility::Inherited,
            enum_token: Default::default(),
            ident: format_ident!("{}Response", item.ident),
            generics: Default::default(),
            brace_token: Default::default(),
            variants: response_variants(&methods),
        }
    )
}

fn trait_service_definition_and_impl_tokens(item: &ItemTrait) -> proc_macro2::TokenStream {
    let trait_ident = &item.ident;
    let trait_service_ident = Ident::new(&format!("{}Service", trait_ident.to_string()), trait_ident.span());
    let request_ident = Ident::new(&format!("{}Request", trait_ident.to_string()), trait_ident.span());
    let response_ident = Ident::new(&format!("{}Response", trait_ident.to_string()), trait_ident.span());

    let matchblocks: Vec<proc_macro2::TokenStream> = item.items.iter()
        .filter_map(|i| match i {
            TraitItem::Method(method) => Some(method),
            _ => None
        })
        .map(
            |method| {
                let method_ident = &method.sig.ident;
                let variant_ident = snake_to_pascal(&method.sig.ident);
                let args: Vec<&Ident> = method.sig.inputs.iter()
                    .filter_map(
                        |arg| match arg {
                            Typed(a) => {
                                match &*a.pat {
                                    Pat::Ident(ident) => {
                                        Some(ident)
                                    }
                                    _ => { panic!("Only 'simple' arguments are supported") }
                                }
                            }
                            _ => None
                        })
                    .map(|arg| { &arg.ident })
                    .collect();
                match method.sig.output {
                    ReturnType::Default => {
                        quote! {
                            #request_ident::#variant_ident{ #(#args),*} => {
                                // locked is the identifier for the local variable used before the match block expansion
                                locked.#method_ident(#(#args),*).await;
                                #response_ident::#variant_ident()
                            }
                        }
                    }
                    ReturnType::Type(_, _) => {
                        quote! {
                            // locked is the identifier for the local variable used before the match block expansion
                            #request_ident::#variant_ident{ #(#args),*} => #response_ident::#variant_ident(locked.#method_ident(#(#args),*).await)
                        }
                    }
                }
            }
        )
        .collect();

    quote! {
        #[derive(Clone)]
        pub struct #trait_service_ident(std::sync::Arc<futures::lock::Mutex<dyn #trait_ident>>);

        impl #trait_service_ident {
            pub fn new(impl_: impl #trait_ident + 'static) -> Self {
                Self { 0: std::sync::Arc::new(futures::lock::Mutex::new(impl_)) }
            }
        }

        impl tower::Service<#request_ident> for #trait_service_ident {
            type Response = #response_ident;
            type Error = ();
            type Future = std::pin::Pin<Box<dyn std::future::Future<Output=Result<Self::Response, Self::Error>> + Send>>;

            fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
                std::task::Poll::Ready(Ok(()))
            }

            fn call(&mut self, req: #request_ident) -> Self::Future {
                let c = self.clone();
                Box::pin(async move {
                    println!("service call future lock");
                    let arc: &std::sync::Arc<futures::lock::Mutex<dyn #trait_ident>> = &c.0;
                    let mut locked = arc.lock().await;
                    let response = match req {
                        #(#matchblocks),*
                    };
                    Ok(response)
                })
            }
        }
    }
}

fn service_impls_trait_tokens(item: &ItemTrait) -> proc_macro2::TokenStream {
    let trait_ident = &item.ident;
    let request_ident = Ident::new(&format!("{}Request", trait_ident.to_string()), trait_ident.span());
    let response_ident = Ident::new(&format!("{}Response", trait_ident.to_string()), trait_ident.span());

    let methods: Vec<proc_macro2::TokenStream> = item.items.iter()
        .filter_map(|i| match i {
            TraitItem::Method(method) => Some(method),
            _ => None
        })
        .map(
            |method| {
                let method_ident = &method.sig.ident;
                let variant_ident = snake_to_pascal(&method.sig.ident);
                let args: Vec<&Ident> = method.sig.inputs.iter()
                    .filter_map(
                        |arg| match arg {
                            Typed(a) => {
                                match &*a.pat {
                                    Pat::Ident(ident) => {
                                        Some(ident)
                                    }
                                    _ => { panic!("Only 'simple' arguments are supported") }
                                }
                            }
                            _ => None
                        })
                    .map(|arg| { &arg.ident })
                    .collect();
                let types: Vec<&Box<Type>> = method.sig.inputs.iter()
                    .filter_map(
                        |arg| match arg {
                            Typed(a) => {
                                match &*a.pat {
                                    Pat::Ident(_) => {
                                        Some(&a.ty)
                                    }
                                    _ => { panic!("Only 'simple' arguments are supported") }
                                }
                            }
                            _ => None
                        })
                    .collect();


                match &method.sig.output {
                    ReturnType::Default => {
                        quote! {
                            async fn #method_ident(&mut self, #(#args: #types),*) {
                                let response = self.call(#request_ident::#variant_ident { #(#args),* }).await.unwrap();
                                match response {
                                    #response_ident::#variant_ident {} => (),
                                    _ => panic!("Invalid response variant")
                                }
                            }
                        }
                    }
                    ReturnType::Type(_, ty) => {
                        quote! {
                          async fn #method_ident(&mut self, #(#args: #types),*) -> #ty {
                                let response = self.call(#request_ident::#variant_ident {#(#args),*}).await.unwrap();
                                match response {
                                    #response_ident::#variant_ident(s) => s,
                                    _ => panic!("Invalid response variant")
                                }
                            }
                        }
                    }
                }
            }
        )
        .collect();

    quote! {
        #[async_trait::async_trait]
        impl<T> #trait_ident for T where T: tower::Service<#request_ident, Response=#response_ident> + Send, T::Error: Debug, T::Future: Send {
            #(#methods)*
        }
    }
}

#[proc_macro_attribute]
pub fn tower_service(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args; // todo: any reasonable arguments to parameterize the generated code?
    let item: ItemTrait = parse_macro_input!(input as ItemTrait);

    let (request_enum, response_enum) = enums(&item);
    let trait_service_definition_and_impl = trait_service_definition_and_impl_tokens(&item);
    let service_impls_trait = service_impls_trait_tokens(&item);

    TokenStream::from(quote! {
        #request_enum // the enum for requests
        #response_enum // the enum for responses

        #[async_trait::async_trait]
        #item // the original trait definition

        #trait_service_definition_and_impl

        #service_impls_trait
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_trait() {
        let t = trybuild::TestCases::new();
        t.pass("tests/01-parse-trait.rs");
    }

    #[test]
    fn test_generate_request_type() {
        let t = trybuild::TestCases::new();
        t.pass("tests/02-generate-request-type.rs");
    }

    #[test]
    fn test_generate_response_type() {
        let t = trybuild::TestCases::new();
        t.pass("tests/03-generate-response-type.rs");
    }

    #[test]
    fn test_generate_service_impl() {
        let t = trybuild::TestCases::new();
        t.pass("tests/04-tower-service-implementation.rs");
    }

    #[test]
    fn test_service_implements_trait() {
        let t = trybuild::TestCases::new();
        t.pass("tests/05-tower-service-implements-trait.rs");
    }
    // #[test]
// fn test_invalid_macro_order() {
//     let t = trybuild::TestCases::new();
//     //t.compile_fail("tests/05-match-expr.rs");
// }
}

