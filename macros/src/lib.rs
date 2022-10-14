use proc_macro::TokenStream;
use proc_macro2::Ident;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
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
            !getset::attribute_contains_any(field, &["ignore", "ignore_get"])
        })
        .map(|field| {
            let field_ident = field
                .ident
                .as_ref()
                .expect("`Getters` only supports structs with named fields");

            let function_ident =
                Ident::new(&format!("get_{}", field_ident), field_ident.span());

            let field_type = &field.ty;

            if field_type
                .to_token_stream()
                .to_string()
                .replace(' ', "")
                .starts_with("::prost::alloc::vec::Vec<")
            {
                quote! {
                    pub fn #function_ident(&mut self) -> ::rhai::Array {
                        self.#field_ident
                            .iter()
                            .map(|x| ::rhai::Dynamic::from(x.clone()))
                            .collect()
                    }
                }
            } else {
                quote! {
                    pub fn #function_ident(&mut self) -> #field_type {
                        self.#field_ident.clone()
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
            !getset::attribute_contains_any(field, &["ignore", "ignore_set"])
        })
        .map(|field| {
            let field_ident = field
                .ident
                .as_ref()
                .expect("`Setters` only supports structs with named fields");

            let function_ident =
                Ident::new(&format!("set_{}", field_ident), field_ident.span());

            let field_type = &field.ty;

            let field_type_name = field_type.to_token_stream().to_string().replace(' ', "");
            if field_type_name.starts_with("::prost::alloc::vec::Vec<") {
                let vec_of_type_name = &field_type_name["::prost::alloc::vec::Vec<".len()..field_type_name.len() - 1];
                let expect_message = format!("Unexpected type in array. Expecting {}", vec_of_type_name);
                let rhai_dynamic_to_rust_fn = match vec_of_type_name {
                    "f64" => quote!(val.as_float().expect(#expect_message)),
                    "f32" => panic!("Protobuf float fields are not supported. AuraeScript only has f64, and f64 cannot be cast to f32 without losing precision"),
                    "i32" => quote!({
                        let val = val.as_int().expect(#expect_message);
                        let val: i32 =::std::convert::TryFrom::<i64>::try_from(val).expect(#expect_message);
                        val
                    }),
                    "i64" => quote!(val.as_int().expect(#expect_message)),
                    "u32" => quote!({
                        let val = val.as_int().expect(#expect_message);
                        let val: u32 =::std::convert::TryFrom::<i64>::try_from(val).expect(#expect_message);
                        val
                    }),
                    "u64" => quote!({
                        let val = val.as_int().expect(#expect_message);
                        let val: u64 =::std::convert::TryFrom::<i64>::try_from(val).expect(#expect_message);
                        val
                    }),
                    "bool" => quote!(val.as_bool().expect(#expect_message)),
                    "::prost::alloc::string::String" => {
                        let expect_message = "Unexpected type in array. Expecting String";
                        quote!(val.into_string().expect(#expect_message)) 
                    },
                    "::prost::alloc::vec::Vec<u8>" => {
                        let expect_message = "Unexpected type in array. Expecting [u8]";
                        quote!(val.into_blob().expect(#expect_message)) 
                    },
                    assuming_proto_message => {
                        let ident = assuming_proto_message
                            .split("::").map(|x| Ident::new(x, field_type.span()));
                        quote!({
                            val.cast::<#(#ident)::*>()
                        })
                    }
                };

                quote! {
                    pub fn #function_ident(&mut self, #field_ident: ::rhai::Array) {
                        let mut vals = vec![];
                        for val in #field_ident.into_iter() {
                            let val = #rhai_dynamic_to_rust_fn;
                            vals.push(val);
                        }
                        self.#field_ident = vals;
                    }
                }
            } else {
                quote! {
                    pub fn #function_ident(&mut self, #field_ident: #field_type) {
                        self.#field_ident = #field_ident;
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
