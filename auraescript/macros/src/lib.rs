use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod get_set;

/// Outputs the macro.
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

/// Creates getter functions for all struct fields.
///
/// Example:
/// ```ignore
/// #[derive(::macros::Getters)]
/// struct MyStruct {
///     field_a: String,
///     #[getset(ignore_get)]
///     field_no_getter: String,
///     #[getset(ignore)]
///     field_not_getter_or_setter: String,
/// }
/// ```
#[proc_macro_derive(Getters, attributes(getset))]
pub fn getters(input: TokenStream) -> TokenStream {
    get_set::getters(input)
}

/// Creates setter functions for all struct fields.
///
/// Example:
/// ```ignore
/// #[derive(::macros::Setters)]
/// struct MyStruct {
///     field_a: String,
///     #[getset(ignore_set)]
///     field_no_setter: String,
///     #[getset(ignore)]
///     field_not_getter_or_setter: String,
/// }
/// ```
#[proc_macro_derive(Setters, attributes(getset))]
pub fn setters(input: TokenStream) -> TokenStream {
    get_set::setters(input)
}
