use heck::ToSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parenthesized, parse_macro_input, Token, Type};

#[allow(clippy::format_push_string)]

pub(crate) fn ops_generator(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as OpsGeneratorInput);

    let OpsGeneratorInput { module: _, name, functions } = input;

    let client_namespace = Ident::new(
        &format!("{}_client", name.to_string().to_snake_case()),
        name.span(),
    );

    let client_ident = Ident::new(&format!("{name}Client"), name.span());

    let op_idents =
        functions.iter().map(|FunctionInput { name: fn_name, .. }| {
            Ident::new(&fn_name.to_string().to_snake_case(), fn_name.span())
        });

    let op_signatures = functions.iter().zip(op_idents.clone()).map(
        |(FunctionInput { name: _, arg, returns }, op_ident)| {
            quote! {
                async fn #op_ident(
                    &self,
                    req: #arg
                ) -> Result<::tonic::Response<#returns>, ::tonic::Status>
            }
        },
    );

    let op_functions = functions
        .iter()
        .zip(op_idents.clone())
        .map(|(FunctionInput { name, arg, returns }, op_ident)| {
            quote! {
                async fn #op_ident(
                    &self,
                    req: #arg
                ) -> Result<::tonic::Response<#returns>, ::tonic::Status> {
                    let mut client = #client_namespace::#client_ident::new(self.channel.clone());
                    client.#name(req).await
                }
            }
        });

    let expanded = quote! {
        #[::tonic::async_trait]
        pub trait #client_ident {
            #(#op_signatures;)*
        }

        #[::tonic::async_trait]
        impl #client_ident for crate::client::AuraeClient {
            #(#op_functions)*
        }
    };

    expanded.into()
}

// TODO: remove allow dead_code
#[allow(dead_code)]
struct OpsGeneratorInput {
    module: Ident,
    name: Ident,
    functions: Punctuated<FunctionInput, Token![,]>,
}

impl Parse for OpsGeneratorInput {
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
