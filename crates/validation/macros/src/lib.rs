/* -------------------------------------------------------------------------- *\
 *                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |              *
 *                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |              *
 *                |  ███████║██║   ██║██████╔╝███████║█████╗   |              *
 *                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |              *
 *                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |              *
 *                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |              *
 *                +--------------------------------------------+              *
 *                                                                            *
 *                         Distributed Systems Runtime                        *
 * -------------------------------------------------------------------------- *
 * Copyright 2022 - 2024, the aurae contributors                              *
 * SPDX-License-Identifier: Apache-2.0                                        *
\* -------------------------------------------------------------------------- */
#![warn(future_incompatible, nonstandard_style, unused)]
#![warn(clippy::unwrap_used)]

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};
use validation::ValidateInput;

mod validation;

// TODO (future-highway): Due to needing to ignore certain tests in CI, we can't format
//    the code in the docs as cargo test will try to test it. Workaround or add the needed
//    deps to this crate to make it pass.
/// Scaffolds validation and provides a `validate` function on the unvalidated type by implementing `validation::ValidatingType`
///
/// # Example
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
///
/// The macro will then generate a trait `MessageTypeValidator` and an empty struct `MessageValidator`.
/// You must `impl MessageTypeValidator for MessageValidator`.
///
/// Decorate fields with the `field_type` attribute, when the unvalidated type differs from the validated type. See example above.
///
/// Optionally, decorate fields with the `validate` attribute to generate default implementations:
/// * `#[validate]` will call `ValidatedFieldType::validate` with the input automatically wrapped in `Some`
/// * `#[validate(opt)]` will call `ValidatedFieldType::validate_optional`
/// * `#[validate(none)]` will pass through the input without performing any validation (input and output type must be the same)
#[proc_macro_derive(ValidatingType, attributes(field_type, validate))]
pub fn validating_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ValidateInput {
        type_ident,
        validated_type_ident,
        field_names,
        field_validations,
        type_validator,
        validator_struct_ident,
    } = input.into();

    let expanded = quote! {
        impl ::validation::ValidatingType<#validated_type_ident> for #type_ident {
            fn validate(
                self,
                parent_name: Option<&str>,
            ) -> ::std::result::Result<
                #validated_type_ident,
                ::validation::ValidationError
            > {
                #validator_struct_ident::pre_validate(
                    &self,
                    parent_name
                )?;

                let Self { #(#field_names,)* } = self;

                #(#field_validations)*

                let output = #validated_type_ident {
                    #(#field_names,)*
                };

                #validator_struct_ident::post_validate(
                    &output,
                    parent_name
                )?;

                Ok(output)
            }
        }

        #type_validator
    };

    expanded.into()
}

/// Same as `ValidatingType`, but the `validation` function is on the validated type.
#[proc_macro_derive(ValidatedType, attributes(field_type, validate))]
pub fn validated_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ValidateInput {
        type_ident,
        validated_type_ident,
        field_names,
        field_validations,
        type_validator,
        validator_struct_ident,
    } = input.into();

    let expanded = quote! {
        impl ::validation::ValidatedType<#type_ident> for #validated_type_ident {
            fn validate(
                input: #type_ident,
                parent_name: Option<&str>,
            ) -> ::std::result::Result<
                #validated_type_ident,
                ::validation::ValidationError
            > {
                #validator_struct_ident::pre_validate(
                    &input,
                    parent_name
                )?;

                let #type_ident { #(#field_names,)* } = input;

                #(#field_validations)*

                let mut output = #validated_type_ident {
                    #(#field_names,)*
                };

                #validator_struct_ident::post_validate(
                    &output,
                    parent_name
                )?;

                Ok(output)
            }
        }

        #type_validator
    };

    expanded.into()
}