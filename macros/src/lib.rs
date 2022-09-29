use proc_macro::{TokenStream};
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(Output)]
pub fn output(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let ident = &ast.ident;

    let expanded = quote! {
        impl #ident {
            pub fn raw(&mut self) {
                println!("{:?}", self);
            }

            pub fn json(&mut self) {
                let serialized = ::serde_json::to_string_pretty(&self).expect("Failed to serialize to pretty json");
                println!("{}", serialized);
            }
        }
    };

    expanded.into()
}