#![warn(future_incompatible, nonstandard_style, unused)]
#![warn(clippy::unwrap_used)]

use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::quote;
use std::str::FromStr;
use syn::{parse_macro_input, Data, DeriveInput, Ident};

/// Scaffolds validation and provides a `validate` function on the unvalidated type by implementing `validation::ValidatingType`
///
/// # Example
/// ```ignore
/// // Given you have this struct:
/// struct Message {
///     cpu_percentage: i32
/// }
///
/// // Create this struct (must be named the same as the unvalidated type with the prefix 'Validated'):
/// #[derive(validation_macros::ValidatingType)]
/// struct ValidatedMessage {
///     #[field_type(i32)]
///     cpu_percentage: u8
/// }
/// ```
///
/// The macro will then generate a trait `MessageTypeValidator` and an empty struct `MessageValidator`.
/// You must `impl MessageTypeValidator for MessageValidator`.
///
/// Decorate fields with the `field_type` attribute, when the unvalidated type differs from the validated type. See example above.
#[proc_macro_derive(ValidatingType, attributes(field_type))]
pub fn validating_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ValidateInput {
        type_ident,
        validated_type_ident,
        field_names,
        field_validations,
        type_validator,
        validator_struct_ident,
    } = parse(input);

    let expanded = quote! {
        impl ::validation::ValidatingType<#validated_type_ident> for #type_ident {
            fn validate(
                self,
                parent_name: Option<&str>,
            ) -> ::std::result::Result<
                #validated_type_ident,
                ::validation::ValidationError
            > {
                #validator_struct_ident::validate_fields(
                    &self,
                    parent_name
                )?;

                let Self { #(#field_names,)* } = self;

                #(#field_validations)*

                Ok(#validated_type_ident {
                    #(#field_names,)*
                })
            }
        }

        #type_validator
    };

    expanded.into()
}

/// Same as `ValidatingType`, but the `validation` function is on the validated type.
#[proc_macro_derive(ValidatedType, attributes(field_type))]
pub fn validated_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ValidateInput {
        type_ident,
        validated_type_ident,
        field_names,
        field_validations,
        type_validator,
        validator_struct_ident,
    } = parse(input);

    let expanded = quote! {
        impl ::validation::ValidatedType<#type_ident> for #validated_type_ident {
            fn validate(
                input: #type_ident,
                parent_name: Option<&str>,
            ) -> ::std::result::Result<
                #validated_type_ident,
                ::validation::ValidationError
            > {
                #validator_struct_ident::validate_fields(
                    &input,
                    parent_name
                )?;

                let #type_ident { #(#field_names,)* } = input;

                #(#field_validations)*

                Ok(#validated_type_ident {
                    #(#field_names,)*
                })
            }
        }

        #type_validator
    };

    expanded.into()
}

struct ValidateInput {
    type_ident: Ident,
    validated_type_ident: Ident,
    field_names: Vec<Ident>,
    field_validations: Vec<proc_macro2::TokenStream>,
    type_validator: proc_macro2::TokenStream,
    validator_struct_ident: Ident,
}

fn parse(
    DeriveInput { ident: validated_type_ident, data, .. }: DeriveInput,
) -> ValidateInput {
    if !validated_type_ident.to_string().starts_with("Validated") {
        panic!("Validated type should be named the same as the unvalidated type with a `Validated` prefix");
    }

    let type_ident = Ident::new(
        &validated_type_ident.to_string().replace("Validated", ""),
        validated_type_ident.span(),
    );

    let validator_trait_ident =
        Ident::new(&format!("{type_ident}TypeValidator"), type_ident.span());

    let validator_struct_ident =
        Ident::new(&format!("{type_ident}Validator"), type_ident.span());

    let validated_type_struct = match data {
        Data::Struct(x) => x,
        _ => panic!("Validated type should be a struct with named fields"),
    };

    let field_names = validated_type_struct
        .fields
        .iter()
        .map(|f| f.ident.as_ref().expect("Expected named field").clone())
        .collect::<Vec<syn::Ident>>();

    let field_validations = field_names
        .iter()
        .map(|field_ident| {
            let field_validation_fn_ident = Ident::new(
                &format!("validate_{}", field_ident),
                field_ident.span(),
            );

            let field_name = field_ident.to_string().to_snake_case();

            quote! {
                let #field_ident = #validator_struct_ident::#field_validation_fn_ident(
                    #field_ident,
                    #field_name,
                    parent_name
                )?;
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    let validator_trait_fns = validated_type_struct
        .fields
        .iter()
        .map(|f| {
            let field_ident = f.ident.as_ref().expect("Expected named field");
            let field_validation_fn_ident = syn::Ident::new(
                &format!("validate_{}", field_ident),
                field_ident.span(),
            );

            let validated_field_type = &f.ty;

            let field_type = f
                .attrs
                .iter()
                .filter(|x| {
                    x.path.segments.len() == 1
                        && x.path.segments[0].ident == "field_type"
                })
                .map(|x| {
                    let arg_type = x.tokens.to_string().replace(['(', ')'], "");

                    syn::Type::Verbatim(
                        proc_macro2::TokenStream::from_str(&arg_type)
                            .expect("Failed to parse field_type value to type"),
                    )
                })
                .collect::<Vec<syn::Type>>();

            let field_type = match field_type.len() {
                0 => &f.ty,
                1 => &field_type[0],
                _ => panic!(
                    "Found {} `field_type` attributes on `{}`. Maximum of 1 is supported.",
                    field_type.len(),
                    field_ident
                )
            };

            quote! {
                fn #field_validation_fn_ident(
                    #field_ident: #field_type,
                    field_name: &str,
                    parent_name: Option<&str>
                ) -> ::std::result::Result<
                    #validated_field_type,
                    ::validation::ValidationError
                >;
            }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    let type_validator = quote! {
        trait #validator_trait_ident {
            #(#validator_trait_fns)*

            fn validate_fields(
                fields: &#type_ident,
                parent_name: Option<&str>
            ) -> ::std::result::Result<(), ::validation::ValidationError> {
                Ok(())
            }
        }

        struct #validator_struct_ident;
    };

    ValidateInput {
        type_ident,
        validated_type_ident,
        field_names,
        field_validations,
        type_validator,
        validator_struct_ident,
    }
}
