use heck::{ToLowerCamelCase, ToSnakeCase};
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, parenthesized, parse_macro_input, Token, Type};

#[allow(clippy::format_push_string)]

pub(crate) fn ops_generator(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as OpsGeneratorInput);

    typescript_generator(&input);

    let OpsGeneratorInput { module, services } = input;

    let output: Vec<(Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>)> = services.iter().map(|ServiceInput { name, functions }| {
        let name_in_snake_case = name.to_string().to_snake_case();
        let name_in_snake_case_ident = Ident::new(&name_in_snake_case, name.span());

        let client_ident = Ident::new(&format!("{}Client", name), name.span());

        let op_idents =
            functions.iter().map(|FunctionInput { name: fn_name, .. }| {
                Ident::new(
                    &format!(
                        "ae__{}__{}__{}",
                        module.to_string().to_snake_case(),
                        name_in_snake_case,
                        fn_name.to_string().to_snake_case(),
                    ),
                    name.span(),
                )
            });

        let op_functions: Vec<proc_macro2::TokenStream> = functions
            .iter()
            .zip(op_idents.clone())
            .map(|(FunctionInput { name, arg, returns }, op_ident)| {
                quote! {
                    #[::deno_core::op]
                    pub(crate) async fn #op_ident(
                        req: ::aurae_proto::#module::#arg
                    ) -> std::result::Result<
                        ::aurae_proto::#module::#returns,
                        ::anyhow::Error
                    > {
                        let client = ::aurae_client::AuraeClient::default().await?;
                        let res = ::aurae_client::#module::#name_in_snake_case_ident::#client_ident::#name(
                            &client,
                            req
                        ).await?;

                        Ok(res.into_inner())
                    }
                }
            })
            .collect();

        let op_decls: Vec<proc_macro2::TokenStream> = op_idents.map(|op_ident| {
            quote! {
                #op_ident::decl()
            }
        }).collect();

        (op_functions, op_decls)
    })
        .collect();

    let op_functions = output.iter().map(|x| &x.0);
    let op_decls = output.iter().map(|x| &x.1);

    let expanded = quote! {
        #(#(#op_functions)*)*

        pub(crate) fn op_decls() -> Vec<::deno_core::OpDecl> {
            vec![#(#(#op_decls,)*)*]
        }
    };

    expanded.into()
}

struct OpsGeneratorInput {
    module: Ident,
    services: Punctuated<ServiceInput, Token![,]>,
}

impl Parse for OpsGeneratorInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let module = input.parse()?;
        let _: Token![,] = input.parse()?;

        let services = input.parse_terminated(ServiceInput::parse)?;

        Ok(Self { module, services })
    }
}

struct ServiceInput {
    name: Ident,
    functions: Punctuated<FunctionInput, Token![,]>,
}

impl Parse for ServiceInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let _ = braced!(content in input);

        let name = content.parse()?;
        let _: Token![,] = content.parse()?;

        let functions = content.parse_terminated(FunctionInput::parse)?;

        Ok(Self { name, functions })
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

fn typescript_generator(input: &OpsGeneratorInput) {
    let OpsGeneratorInput { module, services } = input;

    let services: String = services
        .iter()
        .map(|s| typescript_service_generator(module, s))
        .collect::<Vec<String>>()
        .join("\n\n");

    let gen_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(out_dir) => {
            let mut out_dir = PathBuf::from(out_dir);
            out_dir.push("gen");
            out_dir
        }
        _ => PathBuf::from("gen"),
    };

    let ts_path = {
        let mut out_dir = gen_dir.clone();
        out_dir.push(format!("v0/{module}/{module}.ts"));
        out_dir
    };

    let mut ts =
        OpenOptions::new().read(true).open(ts_path.clone()).unwrap_or_else(
            |_| panic!("protoc output should generate {ts_path:?}"),
        );

    let mut ts_contents = {
        let mut contents = String::new();
        match ts.read_to_string(&mut contents) {
            Ok(0) => panic!("{ts_path:?} is empty"),
            Err(e) => panic!("Failed to read {ts_path:?}: {e}"),
            _ => {}
        };
        contents
    };

    ts_contents.push_str(&services);

    let ts_path = {
        let mut out_dir = gen_dir;
        out_dir.push(format!("{module}.ts"));
        out_dir
    };

    let mut ts = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(ts_path.clone())
        .unwrap_or_else(|_| {
            panic!("Failed to create or overwrite {ts_path:?}")
        });

    write!(ts, "{ts_contents}")
        .unwrap_or_else(|_| panic!("Could not write to {ts_path:?}"));
}

fn typescript_service_generator(
    module: &Ident,
    ServiceInput { name, functions }: &ServiceInput,
) -> String {
    let mut ts_funcs: String =
        format!("export class {name}Client implements {name} {{");

    for FunctionInput { name: fn_name, arg, returns } in functions.iter() {
        let op_name = format!(
            "ae__{}__{}__{}",
            module.to_string().to_snake_case(),
            name.to_string().to_snake_case(),
            fn_name.to_string().to_snake_case()
        );

        let fn_name = fn_name.to_string().to_lower_camel_case();
        let arg = arg.to_token_stream().to_string();
        let returns = returns.to_token_stream().to_string();
        ts_funcs.push_str(&format!(
            r#"
{fn_name}(request: {arg}): Promise<{returns}> {{
    // @ts-ignore
    return Deno.core.ops.{op_name}(request);
}}      
        "#
        ));
    }

    ts_funcs.push('}');
    ts_funcs
}
