/* -------------------------------------------------------------------------- *\
 *                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |              *
 *                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |              *
 *                |  ███████║██║   ██║██████╔╝███████║█████╗   |              *
 *                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |              *
 *                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |              *
 *                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |              *
 *                +--------------------------------------------+              *
 *                                                                            *
 *                         Distributed Systems Runtime                        *
 * -------------------------------------------------------------------------- *
 * Copyright 2022 - 2024, the aurae contributors                              *
 * SPDX-License-Identifier: Apache-2.0                                        *
\* -------------------------------------------------------------------------- */
use heck::ToSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Lit, Path, Token};

#[allow(clippy::format_push_string)]

struct ServiceInput {
    file_path: Lit,
    module: Path,
    service_name: Ident,
}

impl Parse for ServiceInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let file_path: Lit = input.parse()?;
        let _: Token![,] = input.parse()?;
        let module = input.parse()?;
        let _: Token![,] = input.parse()?;
        let service_name = input.parse()?;

        Ok(Self { file_path, module, service_name })
    }
}

pub(crate) fn service(input: TokenStream) -> TokenStream {
    let ServiceInput { file_path, module, service_name } =
        parse_macro_input!(input as ServiceInput);

    let (_, proto) = proto_reader::parse(&file_path);

    let service = proto
        .file_descriptors
        .iter()
        .flat_map(|x| &x.service)
        .find(|x| matches!(x.name(), n if service_name == n))
        .expect("failed to find service");

    let client_namespace = Ident::new(
        &format!("{}_client", service_name.to_string().to_snake_case()),
        service_name.span(),
    );

    let client_ident =
        Ident::new(&format!("{service_name}Client"), service_name.span());

    let fn_name_idents = service.method.iter().map(|m| {
        let fn_name = m.name.as_ref().expect("rpc is missing name");
        Ident::new(&fn_name.to_string().to_snake_case(), file_path.span())
    });

    let rpc_signatures: Vec<_> = service.method
        .iter()
        .zip(fn_name_idents.clone())
        .map(|(m, name)| {
            let input_type = proto_reader::helpers::to_unqualified_type(m.input_type());
            let input_type = Ident::new(input_type, file_path.span());
            let output_type = proto_reader::helpers::to_unqualified_type(m.output_type());
            let output_type = Ident::new(output_type, file_path.span());

            match (m.client_streaming.unwrap_or(false), m.server_streaming.unwrap_or(false)) {
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
                            req: ::proto::#module::#input_type
                        ) -> Result<
                            ::tonic::Response<
                                ::tonic::Streaming<::proto::#module::#output_type>
                            >,
                            ::tonic::Status
                        >
                    }
                }
                _ => {
                    quote! {
                        async fn #name(
                            &self,
                            req: ::proto::#module::#input_type
                        ) -> Result<
                            ::tonic::Response<::proto::#module::#output_type>,
                            ::tonic::Status
                        >
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
                    let mut client = ::proto::#module::#client_namespace::#client_ident::new(self.channel.clone());
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
        impl #client_ident for crate::client::Client {
            #(#rpc_implementations)*
        }
    };

    expanded.into()
}