use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parenthesized, parse_macro_input, Token, Type};
use syn::punctuated::Punctuated;
use heck::ToSnakeCase;

pub(crate) fn client_wrapper(input: TokenStream) -> TokenStream {
    let ClientWrapperInput {
        name,
        functions,
    } = parse_macro_input!(input as ClientWrapperInput);

    let client_namespace = Ident::new(
        &format!("{}_client", name.to_string().to_snake_case()),
        name.span(),
    );

    let client_ident = Ident::new(
        &format!("{}Client", name),
        name.span(),
    );

    let functions = functions
        .into_iter()
        .map(|FunctionInput { name, arg, returns }| {
            quote! {
                pub(crate) fn #name(&mut self, req: #arg) -> #returns {
                    match ::tokio::runtime::Runtime::new() {
                        Ok(rt) => {
                            match rt.block_on(crate::builtin::client::new_client()) {
                                Ok(ch) => {
                                    let mut client = self::#client_namespace::#client_ident::new(ch.channel);
                                    let res = rt.block_on(client.#name(req));
                                    match res {
                                        Ok(x) => {
                                            return x.into_inner();
                                        },
                                        Err(e) => {
                                            eprintln!("{:?}", e);
                                            ::std::process::exit(crate::codes::EXIT_REQUEST_FAILURE);
                                        }
                                    }
                                }
                                Err(e) => {
                                    eprintln!("{:?}", e);
                                    ::std::process::exit(crate::codes::EXIT_CONNECT_FAILURE);
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("{:?}", e);
                            ::std::process::exit(crate::codes::EXIT_RUNTIME_ERROR);
                        }
                    }
                }
            }
        });

    let expanded = quote! {
        #[derive(Debug, Clone)]
        pub struct #name;

        impl Default for #name {
            fn default() -> Self {
                Self {}
            }
        }

        impl #name {
            pub(crate) fn new() -> Self {
                Self::default()
            }

            #(#functions)*
        }
    };

    expanded.into()
}

struct ClientWrapperInput {
    name: Ident,
    functions: Punctuated<FunctionInput, Token![,]>,
}

impl Parse for ClientWrapperInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        let _: Token![,] = input.parse()?;

        let functions = input.parse_terminated(FunctionInput::parse)?;

        Ok(Self {
            name,
            functions,
        })
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

        Ok(Self {
            name,
            arg,
            returns,
        })
    }
}