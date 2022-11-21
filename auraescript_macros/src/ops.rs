use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parenthesized, parse_macro_input, Token, Type};

pub(crate) fn ops_generator(input: TokenStream) -> TokenStream {
    let OpsGeneratorInput { module, name, functions } =
        parse_macro_input!(input as OpsGeneratorInput);

    let mut ts_funcs: String =
        format!("export class {name}Client implements {name} {{");
    for FunctionInput { name: fn_name, arg, returns } in functions.iter() {
        let op_name = format!(
            "ae__{module}__{}__{}",
            name.to_string().to_snake_case(),
            fn_name.to_string().to_snake_case()
        );

        let fn_name = fn_name.to_string().to_upper_camel_case();
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

    let lib_dir = match std::env::var("CARGO_MANIFEST_DIR") {
        Ok(out_dir) => {
            let mut out_dir = PathBuf::from(out_dir);
            out_dir.push("lib");
            out_dir
        }
        _ => PathBuf::from("lib"),
    };

    let ts_path = {
        let mut out_dir = lib_dir.clone();
        out_dir.push(format!("temp/{module}.ts"));
        out_dir
    };

    let mut ts = OpenOptions::new()
        .read(true)
        .open(ts_path.clone())
        .unwrap_or_else(|_| panic!("build.rs should generate {ts_path:?}"));

    let mut ts_contents = {
        let mut contents = String::new();
        match ts.read_to_string(&mut contents) {
            Ok(0) => panic!("{ts_path:?} is empty"),
            Err(e) => panic!("Failed to read {ts_path:?}: {e}"),
            _ => {}
        };
        contents
    };

    ts_contents.push_str(&ts_funcs);

    let ts_path = {
        let mut out_dir = lib_dir;
        out_dir.push(format!("{module}.ts"));
        out_dir
    };

    let mut ts = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(ts_path.clone())
        .unwrap_or_else(|_| {
            panic!("Attempt to create or overwrite {ts_path:?}")
        });

    write!(ts, "{ts_contents}")
        .unwrap_or_else(|_| panic!("Could not write to {ts_path:?}"));

    let client_namespace = Ident::new(
        &format!("{}_client", name.to_string().to_snake_case()),
        name.span(),
    );

    let client_ident = Ident::new(&format!("{}Client", name), name.span());

    let op_idents =
        functions.iter().map(|FunctionInput { name: fn_name, .. }| {
            Ident::new(
                &format!(
                    "ae__{}__{}__{}",
                    module.to_string().to_snake_case(),
                    name.to_string().to_snake_case(),
                    fn_name.to_string().to_snake_case(),
                ),
                name.span(),
            )
        });

    let op_functions = functions
        .iter()
        .zip(op_idents.clone())
        .map(|(FunctionInput { name, arg, returns }, op_ident)| {
            quote! {
                #[::deno_core::op]
                pub(crate) async fn #op_ident(req: #arg) -> Result<#returns, ::anyhow::Error> {
                    let client = crate::builtin::client::AuraeClient::default().await?;
                    let mut client = self::#client_namespace::#client_ident::new(client.channel);
                    let res = client.#name(req).await?;
                    Ok(res.into_inner())
                }
            }
        });

    let op_decls = op_idents.map(|op_ident| {
        quote! {
            #op_ident::decl()
        }
    });

    let expanded = quote! {
        #(#op_functions)*

        pub(crate) fn op_decls() -> Vec<::deno_core::OpDecl> {
            vec![#(#op_decls,)*]
        }
    };

    expanded.into()
}

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
