use heck::ToSnakeCase;
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use std::path::PathBuf;
use std::str::FromStr;
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

    let crate_root = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(dir) => PathBuf::from(dir),
        _ => panic!("env variable 'CARGO_MANIFEST_DIR' was not set. Failed to find crate root"),
    };

    let parsed_file_path = match &file_path {
        Lit::Str(file_path) => {
            let file_path = file_path.value();
            let file_path = file_path.trim_matches('"');

            let file_path = crate_root.join(file_path);

            file_path.canonicalize().unwrap_or_else(|e| {
                panic!(
                    "failed to determine absolute path for {file_path:?}: {e}"
                )
            })
        }
        _ => panic!(
            "expected literal string with path to proto file as first argument"
        ),
    };

    let mut api_dir = parsed_file_path.clone();
    let api_dir = loop {
        match api_dir.parent() {
            Some(parent) => {
                if parent.is_dir() && parent.ends_with("api") {
                    break parent;
                } else {
                    api_dir = parent.to_path_buf();
                }
            }
            _ => panic!("proto file not in api directory"),
        }
    };

    let content = protobuf_parse::Parser::new()
        .protoc()
        .protoc_extra_args(["--experimental_allow_proto3_optional"])
        .include(api_dir)
        .input(&parsed_file_path)
        .parse_and_typecheck()
        .expect("failed to parse proto file");

    let service = content
        .file_descriptors
        .iter()
        .flat_map(|x| &x.service)
        .find(|x| matches!(&x.name, Some(y) if service_name == y))
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

    let rpc_signatures: Vec<_> = service.method.iter().zip(fn_name_idents.clone()).map(
        |(m, name)| {
            let input_type = proc_macro2::TokenStream::from_str(m.input_type
                .as_ref()
                .map(|x| x.split('.').last().expect("input type"))
                .expect("rpc function is missing input type")
            ).expect("rpc input type is not valid");

            let output_type = proc_macro2::TokenStream::from_str(m.output_type
                .as_ref()
                .map(|x| x.split('.').last().expect("output type"))
                .expect("rpc function is missing output type")
            ).expect("rpc output type is not valid");

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
                            req: ::aurae_proto::#module::#input_type
                        ) -> Result<::tonic::Response<::tonic::Streaming<::aurae_proto::#module::#output_type >>, ::tonic::Status>
                    }
                }
                _ => {
                    quote! {
                        async fn #name(
                            &self,
                            req: ::aurae_proto::#module::#input_type
                        ) -> Result<::tonic::Response<::aurae_proto::#module::#output_type>, ::tonic::Status>
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
