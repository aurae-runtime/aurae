/* -------------------------------------------------------------------------- *\
 *             Apache 2.0 License Copyright © 2022 The Aurae Authors          *
 *                                                                            *
 *                +--------------------------------------------+              *
 *                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |              *
 *                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |              *
 *                |  ███████║██║   ██║██████╔╝███████║█████╗   |              *
 *                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |              *
 *                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |              *
 *                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |              *
 *                +--------------------------------------------+              *
 *                                                                            *
 *                         Distributed Systems Runtime                        *
 *                                                                            *
 * -------------------------------------------------------------------------- *
 *                                                                            *
 *   Licensed under the Apache License, Version 2.0 (the "License");          *
 *   you may not use this file except in compliance with the License.         *
 *   You may obtain a copy of the License at                                  *
 *                                                                            *
 *       http://www.apache.org/licenses/LICENSE-2.0                           *
 *                                                                            *
 *   Unless required by applicable law or agreed to in writing, software      *
 *   distributed under the License is distributed on an "AS IS" BASIS,        *
 *   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. *
 *   See the License for the specific language governing permissions and      *
 *   limitations under the License.                                           *
 *                                                                            *
\* -------------------------------------------------------------------------- */

use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use std::str::FromStr;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Data, DeriveInput, Field, Token, Type};

pub(crate) fn getters(input: TokenStream) -> TokenStream {
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
            !GetSetAttribute::contains_any(field, &["ignore", "ignore_get"])
        })
        .map(|field| {
            let field_ident = field
                .ident
                .as_ref()
                .expect("`Getters` only supports structs with named fields");

            let function_ident =
                Ident::new(&format!("get_{}", field_ident), field_ident.span());

            let field_type = &field.ty;
            assert_field_type_is_supported(field_type);

            if try_field_type_as_vec_type_name(field_type).is_some() {
                quote! {
                    pub fn #function_ident(&mut self) -> ::rhai::Array {
                        self.#field_ident
                            .iter()
                            .map(|x| ::rhai::Dynamic::from(x.clone()))
                            .collect()
                    }
                }
            } else {
                let value_as_rhai_field_type =
                    match &*field_type_to_name(field_type) {
                        "f64" | "bool" => {
                            quote!(::rhai::Dynamic::from(self.#field_ident))
                        }
                        "i32" | "i64" | "u32" => {
                            quote!(::rhai::Dynamic::from(self.#field_ident as i64))
                        }
                        other if other.starts_with("::core::option::Option<::prost::alloc::boxed::Box<") => {
                            quote! {
                                match &self.#field_ident {
                                    Some(#field_ident) => rhai::Dynamic::from((**#field_ident).clone()),
                                    None => rhai::Dynamic::UNIT,
                                }
                            }
                        }
                        other if other.starts_with("::core::option::Option<") => {
                            quote! {
                                match &self.#field_ident {
                                    Some(#field_ident) => rhai::Dynamic::from((*#field_ident).clone()),
                                    None => ::rhai::Dynamic::UNIT,
                                }
                            }
                        }
                        _ => {
                            quote!(::rhai::Dynamic::from(self.#field_ident.clone()))
                        }
                    };

                quote! {
                    pub fn #function_ident(&mut self) -> ::rhai::Dynamic {
                        #value_as_rhai_field_type
                    }
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

pub(crate) fn setters(input: TokenStream) -> TokenStream {
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
            !GetSetAttribute::contains_any(field, &["ignore", "ignore_set"])
        })
        .map(|field| {
            let field_ident = field
                .ident
                .as_ref()
                .expect("`Setters` only supports structs with named fields");

            let function_ident =
                Ident::new(&format!("set_{}", field_ident), field_ident.span());

            let field_type = &field.ty;
            assert_field_type_is_supported(field_type);

            if let Some(vec_of_type_name) = try_field_type_as_vec_type_name(field_type) {
                let val_as_rust_field_type = expression_for_val_as_rust_field_type(
                    field_type,
                    &format!("Unexpected type in array. Expecting {}", vec_of_type_name)
                );

                quote! {
                    pub fn #function_ident(&mut self, #field_ident: ::rhai::Array) {
                        self.#field_ident = #field_ident
                            .into_iter()
                            .map(|val| #val_as_rust_field_type)
                            .collect();
                    }
                }
            } else {
                let val_as_rust_field_type = expression_for_val_as_rust_field_type(
                    field_type,
                    &format!("Unexpected type. Expecting {}", field_type_to_name(field_type))
                );

                quote! {
                    pub fn #function_ident(&mut self, val: ::rhai::Dynamic) {
                        self.#field_ident = #val_as_rust_field_type;
                    }
                }
            }
        })
        .collect();

    let expanded = quote! {
        impl #ident {
            #(#field_setters)*
        }
    };

    return expanded.into();

    fn expression_for_val_as_rust_field_type(
        field_type: &Type,
        expect_message: &str,
    ) -> proc_macro2::TokenStream {
        let field_type_name = if let Some(vec_of_field_type_name) =
            try_field_type_as_vec_type_name(field_type)
        {
            vec_of_field_type_name
        } else {
            field_type_to_name(field_type)
        };

        // val is of type rhai::Dynamic
        match &*field_type_name {
            "f64" => quote!(val.as_float().expect(#expect_message)),
            "i64" => quote!(val.as_int().expect(#expect_message)),
            "i32" | "u32" => quote!({
                let val = val.as_int().expect(#expect_message);
                ::std::convert::TryFrom::<i64>::try_from(val).expect(#expect_message)
            }),
            "bool" => quote!(val.as_bool().expect(#expect_message)),
            "String" => quote!(val.into_string().expect(#expect_message)),
            "Vec<u8>" => quote!(val.into_blob().expect(#expect_message)),
            assuming_proto_message => {
                if assuming_proto_message.starts_with(
                    "::core::option::Option<::prost::alloc::boxed::Box<",
                ) {
                    let field_type = proto_message_name_to_type(&assuming_proto_message
                        ["::core::option::Option<::prost::alloc::boxed::Box<".len()
                        ..assuming_proto_message.len() - 2]);

                    quote!({
                        match val.as_unit() {
                            Ok(_) => None,
                            Err(_) => Some(::std::boxed::Box::new(val.cast::<#field_type>())),
                        }
                    })
                } else if assuming_proto_message
                    .starts_with("::core::option::Option<")
                {
                    let field_type = proto_message_name_to_type(
                        &assuming_proto_message["::core::option::Option<".len()
                            ..assuming_proto_message.len() - 1],
                    );

                    quote!({
                        match val.as_unit() {
                            Ok(_) => None,
                            Err(_) => Some(val.cast::<#field_type>()),
                        }
                    })
                } else {
                    let field_type =
                        proto_message_name_to_type(assuming_proto_message);

                    quote!(val.cast::<#field_type>())
                }
            }
        }
    }

    fn proto_message_name_to_type(name: &str) -> proc_macro2::TokenStream {
        proc_macro2::TokenStream::from_str(name)
            .unwrap_or_else(|_| panic!("Could not parse type `{}`", name))
    }
}

fn field_type_to_name(field_type: &Type) -> String {
    let field_type_name: String = field_type
        .to_token_stream()
        .to_string()
        .chars()
        .filter(|x| !x.is_whitespace())
        .collect();

    match &*field_type_name {
        "::prost::alloc::string::String" => "String".to_string(),
        "::prost::alloc::vec::Vec<u8>" => "Vec<u8>".to_string(),
        _ => field_type_name,
    }
}

fn try_field_type_as_vec_type_name(field_type: &Type) -> Option<String> {
    let field_type_name = field_type_to_name(field_type);
    if field_type_name.starts_with("::prost::alloc::vec::Vec<") {
        let field_type_name = match &field_type_name
            ["::prost::alloc::vec::Vec<".len()..field_type_name.len() - 1]
        {
            "::prost::alloc::string::String" => "String",
            "::prost::alloc::vec::Vec<u8>" => "Vec<u8>",
            other => other,
        };

        Some(field_type_name.to_string())
    } else {
        None
    }
}

fn assert_field_type_is_supported(field_type: &Type) {
    let field_type_name = if let Some(field_type_name) =
        try_field_type_as_vec_type_name(field_type)
    {
        field_type_name
    } else {
        field_type_to_name(field_type)
    };

    match &*field_type_name {
        "f32" => panic!("Protobuf float fields are not supported. AuraeScript only has f64, and f64 cannot be cast to f32 without losing precision"),
        "u64" => panic!("Protobuf uint64 and fixed64 fields are not supported. AuraeScript only has i64, and u64 cannot be cast to i64 safely"),
        _ => {}
    };
}

struct GetSetAttribute {
    options: Punctuated<Ident, Token![,]>,
}

impl GetSetAttribute {
    fn contains_any(field: &Field, values: &[&str]) -> bool {
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

impl Parse for GetSetAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let options = input.parse_terminated(Ident::parse)?;
        Ok(GetSetAttribute { options })
    }
}
