use heck::ToSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parenthesized, parse_macro_input, Path, Token, Type};

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
        |(FunctionInput { name: _, arg, returns }, name)| {
            quote! {
                async fn #name(
                    &self,
                    req: ::aurae_proto::#module::#arg
                ) -> Result<::tonic::Response<::aurae_proto::#module::#returns>, ::tonic::Status>
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
    arg: Type,
    returns: Type,
}

impl Parse for FunctionInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;

        let content;
        let _ = parenthesized!(content in input);
        let arg = content.parse()?;

        let _: Token![->] = input.parse()?;

        let returns = input.parse()?;

        Ok(Self { name, arg, returns })
    }
}
