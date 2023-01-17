use heck::ToSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{bracketed, parenthesized, parse_macro_input, Path, Token, Type};

#[allow(clippy::format_push_string)]

pub(crate) fn service(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ServiceInput);

    let ServiceInput { module, name, functions } = input;

    let client_namespace = Ident::new(
        &format!("{}_client", name.to_string().to_snake_case()),
        name.span(),
    );

    let client_ident = Ident::new(&format!("{name}Client"), name.span());

    let fn_name_idents =
        functions.iter().map(|FunctionInput { name: fn_name, .. }| {
            Ident::new(&fn_name.to_string().to_snake_case(), fn_name.span())
        });

    let rpc_signatures: Vec<_> = functions.iter().zip(fn_name_idents.clone()).map(
        |(FunctionInput { name: _, client_streaming, arg, server_streaming, returns }, name)| {
            match (client_streaming, server_streaming) {
                (true, true) => {
                    todo!("bidirectional streaming")
                }
                (true, false) => {
                    todo!("client streaming")
                },
                (false, true) => {
                    quote! {
                        async fn #name(
                            &self,
                            req: ::aurae_proto::#module::#arg
                        ) -> Result<::tonic::Response<::tonic::Streaming<::aurae_proto::#module::#returns>>, ::tonic::Status>
                    }
                }
                _ => {
                    quote! {
                        async fn #name(
                            &self,
                            req: ::aurae_proto::#module::#arg
                        ) -> Result<::tonic::Response<::aurae_proto::#module::#returns>, ::tonic::Status>
                    }
                }
            }
        },
    ).collect();

    let rpc_implementations: Vec<_> = rpc_signatures
        .iter()
        .zip(fn_name_idents)
        .map(|(signature, name)| {
            quote! {
                #signature {
                    let mut client = ::aurae_proto::#module::#client_namespace::#client_ident::new(self.channel.clone());
                    client.#name(req).await
                }
            }
        }).collect();

    let expanded = quote! {
        #[::tonic::async_trait]
        pub trait #client_ident {
            #(#rpc_signatures;)*
        }

        #[::tonic::async_trait]
        impl #client_ident for crate::client::AuraeClient {
            #(#rpc_implementations)*
        }
    };

    expanded.into()
}

struct ServiceInput {
    module: Path,
    name: Ident,
    functions: Punctuated<FunctionInput, Token![,]>,
}

impl Parse for ServiceInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let module = input.parse()?;
        let _: Token![,] = input.parse()?;

        let name = input.parse()?;
        let _: Token![,] = input.parse()?;

        let functions = input.parse_terminated(FunctionInput::parse)?;

        Ok(Self { module, name, functions })
    }
}

struct FunctionInput {
    name: Ident,
    client_streaming: bool,
    arg: Type,
    server_streaming: bool,
    returns: Type,
}

impl Parse for FunctionInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;

        let content;
        let _ = parenthesized!(content in input);
        let (client_streaming, arg) = if content.peek(syn::token::Bracket) {
            let content2;
            let _ = bracketed!(content2 in content);
            (true, content2.parse()?)
        } else {
            (false, content.parse()?)
        };

        let _: Token![->] = input.parse()?;

        let (server_streaming, returns) = if input.peek(syn::token::Bracket) {
            let content;
            let _ = bracketed!(content in input);
            (true, content.parse()?)
        } else {
            (false, input.parse()?)
        };

        Ok(Self { name, client_streaming, arg, server_streaming, returns })
    }
}
