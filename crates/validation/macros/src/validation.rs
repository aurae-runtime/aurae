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
use heck::ToSnakeCase;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use std::str::FromStr;
use syn::{Data, DeriveInput};

enum AutoValidate {
    No,
    Validate,
    ValidateOpt,
    ValidateNone,
    ValidateForCreation,
}

pub(crate) struct ValidateInput {
    pub type_ident: Ident,
    pub validated_type_ident: Ident,
    pub field_names: Vec<Ident>,
    pub field_validations: Vec<TokenStream>,
    pub type_validator: TokenStream,
    pub validator_struct_ident: Ident,
}

impl From<DeriveInput> for ValidateInput {
    fn from(input: DeriveInput) -> Self {
        let DeriveInput { ident: validated_type_ident, data, .. } = input;

        if !validated_type_ident.to_string().starts_with("Validated") {
            panic!("Validated type should be named the same as the unvalidated type with a `Validated` prefix");
        }

        let type_ident = Ident::new(
            &validated_type_ident.to_string().replace("Validated", ""),
            validated_type_ident.span(),
        );

        let validator_trait_ident = Ident::new(
            &format!("{type_ident}TypeValidator"),
            type_ident.span(),
        );

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
            .collect::<Vec<_>>();

        let field_validations = field_names
            .iter()
            .map(|field_ident| {
                let field_validation_fn_ident = Ident::new(
                    &format!("validate_{field_ident}"),
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
            .collect::<Vec<_>>();

        let validator_trait_fns = validated_type_struct
            .fields
            .iter()
            .map(|f| {
                let field_ident = f.ident.as_ref().expect("Expected named field");
                let field_validation_fn_ident = Ident::new(
                    &format!("validate_{field_ident}"),
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
                            TokenStream::from_str(&arg_type)
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

                let auto_validate = f.attrs
                    .iter()
                    .filter(|x| {
                        x.path.segments.len() == 1 && x.path.segments[0].ident == "validate"
                    })
                    .map(|attr| {
                        let arg = attr.tokens.to_string().replace(['(', ')'], "");
                        match &*arg {
                            "" => AutoValidate::Validate,
                            "opt" => AutoValidate::ValidateOpt,
                            "none" => AutoValidate::ValidateNone,
                            "create" => AutoValidate::ValidateForCreation,
                            _=> panic!("`opt`, `none`, and `create` are a valid args for the `validate` attribute"),
                        }
                    })
                    .next()
                    .or(Some(AutoValidate::No))
                    .expect("auto_validate");

                let base = quote! {
                    fn #field_validation_fn_ident(
                        #field_ident: #field_type,
                        field_name: &str,
                        parent_name: Option<&str>
                    ) -> ::std::result::Result<
                        #validated_field_type,
                        ::validation::ValidationError
                    >
                };

                match auto_validate {
                    AutoValidate::No => quote! {
                        #base;
                    },
                    AutoValidate::Validate => quote! {
                        #base {
                            validation::ValidatedField::validate(Some(#field_ident), field_name, parent_name)
                        }
                    },
                    AutoValidate::ValidateOpt => quote! {
                        #base {
                            validation::ValidatedField::validate_optional(#field_ident, field_name, parent_name)
                        }
                    },
                    AutoValidate::ValidateNone => quote! {
                        #base {
                            Ok(#field_ident)
                        }
                    },
                    AutoValidate::ValidateForCreation => quote! {
                        #base {
                            validation::ValidatedField::validate_for_creation(Some(#field_ident), field_name, parent_name)
                        }
                    },
                }
            })
            .collect::<Vec<_>>();

        let type_validator = quote! {
            trait #validator_trait_ident {
                #(#validator_trait_fns)*

                fn pre_validate(
                    input: &#type_ident,
                    parent_name: Option<&str>
                ) -> ::std::result::Result<(), ::validation::ValidationError> {
                    Ok(())
                }

                fn post_validate(
                    output: &#validated_type_ident,
                    parent_name: Option<&str>
                ) -> ::std::result::Result<(), ::validation::ValidationError> {
                    Ok(())
                }
            }

            struct #validator_struct_ident;
        };

        Self {
            type_ident,
            validated_type_ident,
            field_names,
            field_validations,
            type_validator,
            validator_struct_ident,
        }
    }
}