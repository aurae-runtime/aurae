use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

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

#[proc_macro_derive(Getters, attributes(getset))]
pub fn getters(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let ident = &ast.ident;

    let data_struct = match ast.data {
        Data::Struct(data_struct) => data_struct,
        _ => panic!("`Getters` only supports structs"),
    };

    let field_getters: Vec<proc_macro2::TokenStream> = data_struct
        .fields
        .iter()
        .filter(|field| {
            !getset::attribute_contains_any(*field, &["ignore", "ignore_get"])
        })
        .map(|field| {
            let field_ident = field
                .ident
                .as_ref()
                .expect("`Getters` only supports structs with named fields");

            let function_ident =
                Ident::new(&format!("get_{}", field_ident), field_ident.span());

            let field_type = &field.ty;

            quote! {
                pub fn #function_ident(&mut self) -> #field_type {
                    self.#field_ident.clone()
                }
            }
        })
        .collect();

    let expanded = quote! {
        impl #ident {
            #(#field_getters)*
        }
    };

    expanded.into()
}

#[proc_macro_derive(Setters, attributes(getset))]
pub fn setters(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let ident = &ast.ident;

    let data_struct = match ast.data {
        Data::Struct(data_struct) => data_struct,
        _ => panic!("`Setters` only supports structs"),
    };

    let field_setters: Vec<proc_macro2::TokenStream> = data_struct
        .fields
        .iter()
        .filter(|field| {
            !getset::attribute_contains_any(*field, &["ignore", "ignore_set"])
        })
        .map(|field| {
            let field_ident = field
                .ident
                .as_ref()
                .expect("`Setters` only supports structs with named fields");

            let function_ident =
                Ident::new(&format!("set_{}", field_ident), field_ident.span());

            let field_type = &field.ty;

            quote! {
                pub fn #function_ident(&mut self, #field_ident: #field_type) {
                    self.#field_ident = #field_ident;
                }
            }
        })
        .collect();

    let expanded = quote! {
        impl #ident {
            #(#field_setters)*
        }
    };

    expanded.into()
}

mod getset {
    use proc_macro2::Ident;
    use syn::parse::{Parse, ParseStream};
    use syn::punctuated::Punctuated;
    use syn::{Field, Token};

    struct GetSetAttribute {
        options: Punctuated<Ident, Token![,]>,
    }

    impl Parse for GetSetAttribute {
        fn parse(input: ParseStream) -> syn::Result<Self> {
            let options = input.parse_terminated(Ident::parse)?;
            Ok(GetSetAttribute { options })
        }
    }

    pub(crate) fn attribute_contains_any(
        field: &Field,
        values: &[&str],
    ) -> bool {
        field
            .attrs
            .iter()
            .filter(|attribute| {
                let seg = match attribute.path.segments.len() {
                    1 => &attribute.path.segments[0],
                    2 if attribute.path.segments[0].ident == "macros" => {
                        &attribute.path.segments[1]
                    }
                    _ => {
                        return false;
                    }
                };

                seg.ident == "getset"
            })
            .any(|attribute| {
                let GetSetAttribute { options } = attribute
                    .parse_args_with(GetSetAttribute::parse)
                    .expect("failed to parse `getset` attribute");

                options
                    .into_iter()
                    .any(|option| values.iter().any(|v| option == v))
            })
    }
}
