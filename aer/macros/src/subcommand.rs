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
use itertools::Itertools;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use protobuf::descriptor::field_descriptor_proto::Label::LABEL_REPEATED;
use protobuf::descriptor::field_descriptor_proto::Type;
use protobuf::descriptor::{
    DescriptorProto, FieldDescriptorProto, MethodDescriptorProto,
};
use protobuf_parse::ParsedAndTypechecked;
use quote::{ToTokens, quote};
use std::collections::VecDeque;
use std::str::FromStr;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::token::{Brace, Token};
use syn::{Lit, Path, Token, braced, bracketed, parse_macro_input};

struct SubcommandInput {
    file_path: Lit,
    module: Path,
    service_name: Ident,
    commands: Option<Punctuated<CommandInput, Token![,]>>,
}

impl Parse for SubcommandInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let file_path: Lit = input.parse()?;
        let _: Token![,] = input.parse()?;
        let module = input.parse()?;
        let _: Token![,] = input.parse()?;
        let service_name = input.parse()?;

        let commands = if input.peek(Token![,]) {
            let _: Token![,] = input.parse()?;
            Some(input.parse_terminated(CommandInput::parse)?)
        } else {
            None
        };

        Ok(Self { file_path, module, service_name, commands })
    }
}

struct CommandInput {
    name: Ident,
    flags: Option<Punctuated<FlagInput, Token![,]>>,
}

impl Parse for CommandInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;

        let flags = if Brace::peek(input.cursor()) {
            let content;
            let _ = braced!(content in input);
            Some(content.parse_terminated(FlagInput::parse)?)
        } else {
            None
        };

        Ok(Self { name, flags })
    }
}

struct FlagInput {
    name: Ident,
    attribute: proc_macro2::TokenStream,
}

impl Parse for FlagInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;

        let content;
        let _ = bracketed!(content in input);
        let attribute = content.parse()?;

        Ok(Self { name, attribute })
    }
}

pub fn subcommand(input: TokenStream, panic_on_issue: bool) -> TokenStream {
    let SubcommandInput { file_path, module, service_name, commands } =
        parse_macro_input!(input);

    let file_path_span = file_path.span();

    let (_file_path, proto) = proto_reader::parse(&file_path);

    let command_ident =
        Ident::new(&format!("{service_name}Commands"), file_path_span);

    let service = proto
        .file_descriptors
        .iter()
        .flat_map(|f| &f.service)
        .find(|s| matches!(s.name(), n if service_name == n))
        .expect("failed to find gRPC service");

    let commands: Vec<_> = service
        .method
        .iter()
        .map(|m| {
            let method_name = m.name();

            let command = commands
                .as_ref()
                .map(|cs| cs.iter().find(|c| c.name == method_name))
                .unwrap_or(None);

            let input_type =
                proto_reader::helpers::to_unqualified_type(m.input_type());
            let input_type_message = proto
                .file_descriptors
                .iter()
                .flat_map(|f| &f.message_type)
                .find(|m| matches!(m.name(), n if input_type == n))
                .unwrap_or_else(|| {
                    panic!("failed to find message '{input_type}'")
                });

            if input_type_message.field.is_empty() {
                Command {
                    module: &module,
                    service_name: &service_name,
                    method: m,
                    fields: vec![],
                }
            } else {
                let fields: Vec<_> = resolve_fields(
                    file_path_span,
                    &proto,
                    input_type_message,
                    panic_on_issue,
                )
                .into_iter()
                .map(|mut f| {
                    let attribute = command
                        .map(|c| {
                            c.flags.as_ref().map(|flags| {
                                flags.iter().find_map(|flag| {
                                    if f.get_resolved_field_ident() == flag.name
                                    {
                                        Some(&flag.attribute)
                                    } else {
                                        None
                                    }
                                })
                            })
                        })
                        .unwrap_or(None)
                        .unwrap_or(None)
                        .map_or_else(
                            || quote! { #[arg(long)] },
                            |t| quote! { #[arg(#t)]},
                        );

                    f.attribute = attribute;
                    f
                })
                .collect();

                Command {
                    module: &module,
                    service_name: &service_name,
                    method: m,
                    fields,
                }
            }
        })
        .collect();

    let command_variants = {
        let variants = commands.iter().map(|c| c.to_variant());
        quote! {
            #[derive(Debug, ::clap::Subcommand)]
            pub enum #command_ident {
                #(#variants,)*
            }
        }
    };

    let impls =
        commands.into_iter().map(|c| c.into_impl(&proto, panic_on_issue));

    let expanded = quote! {
        #command_variants

        impl #command_ident {
            pub async fn execute(self) -> ::anyhow::Result<()> {
                match self {
                    #(#impls)*
                }
            }
        }
    };

    expanded.into()
}

struct Command<'a> {
    module: &'a Path,
    service_name: &'a Ident,
    method: &'a MethodDescriptorProto,
    fields: Vec<ResolvedField>,
}

impl Command<'_> {
    fn to_variant(&self) -> proc_macro2::TokenStream {
        let Self { module: _, service_name: _, method, fields } = self;

        let method_ident = Ident::new(method.name(), Span::call_site());

        if self.fields.is_empty() {
            quote! {
                #[command()]
                #method_ident
            }
        } else {
            let fields = fields.iter().map(|f| f.to_variant());

            quote! {
                #[command(arg_required_else_help = true)]
                #method_ident {
                    #(#fields,)*
                }
            }
        }
    }

    fn into_impl(
        self,
        proto: &ParsedAndTypechecked,
        panic_on_issue: bool,
    ) -> proc_macro2::TokenStream {
        let Self { module, service_name, method, fields } = self;

        let method_ident = Ident::new(method.name(), Span::call_site());

        let command_fields: Vec<_> =
            fields.iter().map(|f| f.get_resolved_field_ident()).collect();

        // Mapping is hard. Let's just "write" the code.
        let mapping = write_mapping(module, proto, method, panic_on_issue);
        let mapping =
            proc_macro2::TokenStream::from_str(&mapping).expect("mapping");

        let client_mod = Ident::new(
            &service_name.to_string().to_snake_case(),
            service_name.span(),
        );

        let client_ident =
            Ident::new(&format!("{service_name}Client"), service_name.span());

        let function =
            Ident::new(&method.name().to_snake_case(), Span::call_site());

        let execute = match (
            method.client_streaming(),
            method.server_streaming(),
        ) {
            (true, true) => {
                todo!("bidirectional streaming")
            }
            (true, false) => {
                todo!("client streaming")
            }
            (false, true) => quote! {
                crate::execute_server_streaming!(::client::#module::#client_mod::#client_ident::#function, req);
            },
            _ => quote! {
                let _ = crate::execute!(::client::#module::#client_mod::#client_ident::#function, req);
            },
        };

        quote! {
            Self::#method_ident {
                #(#command_fields),*
            } => {
                let req = #mapping
                #execute
                Ok(())
            }
        }
    }
}

enum FieldType {
    Primitive,
    Message,
    Map,
    VecPrimitive,
    VecMessage,
}

impl FieldType {
    fn resolve(field: &FieldDescriptorProto, panic_on_issue: bool) -> Self {
        let is_repeated =
            matches!(field.label, Some(l) if l == LABEL_REPEATED.into());

        if matches!(field.type_(), Type::TYPE_MESSAGE) {
            if is_repeated {
                let name = field.type_name();
                if name.ends_with("Entry") {
                    if panic_on_issue {
                        panic!(
                            "Map not supported by the macro. To generate code that is close to correct, use the `subcommand_for_dev_only` macro. The code will have compilation errors, but you can expand the macro and save some typing"
                        );
                    }
                    Self::Map
                } else {
                    Self::VecMessage
                }
            } else {
                Self::Message
            }
        } else if is_repeated {
            Self::VecPrimitive
        } else {
            Self::Primitive
        }
    }
}

struct ResolvedField {
    attribute: proc_macro2::TokenStream,
    field_ident: VecDeque<Ident>,
    type_ident: proc_macro2::TokenStream,
}

impl ResolvedField {
    fn get_resolved_field_ident(&self) -> Ident {
        self.field_ident
            .iter()
            .map(|i| i.to_string())
            .reduce(|mut a, b| {
                a.push('_');
                a.push_str(&b);
                a
            })
            .map(|i| Ident::new(&i, self.field_ident[0].span()))
            .expect("resolved field_ident")
    }

    fn to_variant(&self) -> proc_macro2::TokenStream {
        let field_ident = self.get_resolved_field_ident();
        let Self { attribute, field_ident: _, type_ident } = self;

        quote! {
            #attribute
            #field_ident: #type_ident
        }
    }
}

fn resolve_fields<'a>(
    span: Span,
    proto: &'a ParsedAndTypechecked,
    message: &'a DescriptorProto,
    panic_on_issue: bool,
) -> Vec<ResolvedField> {
    message
        .field
        .iter()
        .flat_map(|f| {
            let field_ident = Ident::new(f.name(), span);

            match FieldType::resolve(f, panic_on_issue) {
                FieldType::Primitive | FieldType::VecPrimitive => {
                    let type_ident =
                        proto_reader::helpers::to_rust_type(f.type_(), span);

                    let type_ident = if f.proto3_optional() {
                        quote! { Option<#type_ident> }
                    } else {
                        quote! { #type_ident }
                    };

                    vec![ResolvedField {
                        attribute: quote! { #[arg(long)] },
                        field_ident: vec![field_ident].into(),
                        type_ident,
                    }]
                }
                FieldType::Message | FieldType::VecMessage => {
                    let message = proto_reader::helpers::find_message(
                        proto,
                        proto_reader::helpers::to_unqualified_type(
                            f.type_name(),
                        ),
                    )
                    .unwrap_or_else(|| {
                        panic!(
                            "failed to find message '{}' from field {f:#?}",
                            f.type_name()
                        )
                    });

                    resolve_fields(span, proto, message, panic_on_issue)
                        .into_iter()
                        .map(|mut f| {
                            f.field_ident.push_front(field_ident.clone());
                            f
                        })
                        .collect()
                }
                FieldType::Map => {
                    vec![]
                }
            }
        })
        .collect()
}

fn write_mapping(
    module: &Path,
    proto: &ParsedAndTypechecked,
    method: &MethodDescriptorProto,
    panic_on_issue: bool,
) -> String {
    fn write_value_from_field(
        command_field_parts: &mut VecDeque<String>,
        mapping: &mut String,
        field: &FieldDescriptorProto,
        panic_on_issue: bool,
    ) {
        let field_type = FieldType::resolve(field, panic_on_issue);
        if let FieldType::VecPrimitive = field_type {
            mapping.push_str("vec![");
        }

        mapping.push_str(&command_field_parts.iter().join("_"));

        if let FieldType::VecPrimitive = field_type {
            mapping.push(']');
        }

        mapping.push(',');
    }

    /// Generates code to construct a nested message type from CLI arguments.
    ///
    /// For `Message` types: generates `Some(MessageType { field: value, ... })`
    /// For `VecMessage` types (repeated messages): generates either:
    ///   - `vec![MessageType { ... }]` if values are provided, OR
    ///   - `vec![]` if the first string field is empty (see below)
    ///
    /// ## VecMessage Empty Check
    ///
    /// Proto `repeated` message fields (e.g., `repeated DriveMount drive_mounts`)
    /// are flattened into individual CLI args with default values. Without special
    /// handling, this creates a problem:
    ///
    /// ```text
    /// // Proto definition:
    /// message VirtualMachine {
    ///     repeated DriveMount drive_mounts = 7;
    /// }
    /// message DriveMount {
    ///     string image_path = 1;  // CLI: --machine-drive-mounts-image-path ""
    ///     string vm_path = 2;     // CLI: --machine-drive-mounts-vm-path ""
    ///     ...
    /// }
    /// ```
    ///
    /// Without the empty check, the generated code would be:
    /// ```text
    /// drive_mounts: vec![DriveMount { image_path: "", vm_path: "", ... }]
    /// ```
    ///
    /// This creates a vec with ONE invalid entry (empty paths), causing the server
    /// to fail when it tries to use the empty path. The fix generates conditional
    /// code that checks if the first string field is empty:
    ///
    /// ```text
    /// drive_mounts: if machine_drive_mounts_image_path.is_empty() {
    ///     vec![]  // No drive mounts specified - empty vec is correct
    /// } else {
    ///     vec![DriveMount { image_path: machine_drive_mounts_image_path, ... }]
    /// }
    /// ```
    ///
    /// This allows optional repeated message fields to work correctly when the
    /// user doesn't provide values via CLI - they get an empty vec instead of
    /// a vec containing an invalid/empty message.
    fn write_value_from_type(
        module_path: &str,
        proto: &ParsedAndTypechecked,
        command_field_parts: &mut VecDeque<String>,
        mapping: &mut String,
        field: &FieldDescriptorProto,
        panic_on_issue: bool,
    ) {
        let field_type = FieldType::resolve(field, panic_on_issue);

        let field_type_name =
            proto_reader::helpers::to_unqualified_type(field.type_name());

        let field_type_message =
            proto_reader::helpers::find_message(proto, field_type_name)
                .expect("failed to find message for field");

        // For VecMessage types, find the first non-repeated string field.
        // This field will be used to determine if the user provided any values.
        // If it's empty (default), we generate an empty vec instead of a vec
        // with one invalid/empty message entry.
        let first_string_field = if matches!(field_type, FieldType::VecMessage)
        {
            field_type_message.field.iter().find(|f| {
                matches!(f.type_(), Type::TYPE_STRING)
                    && !matches!(f.label, Some(l) if l == LABEL_REPEATED.into())
            })
        } else {
            None
        };

        match field_type {
            FieldType::VecMessage => {
                if let Some(check_field) = first_string_field {
                    // Generate conditional: check if the first string field is empty.
                    // Build the full CLI arg name (e.g., "machine_drive_mounts_image_path")
                    let check_field_name = check_field.name();
                    let mut check_path = command_field_parts.iter().join("_");
                    if !check_path.is_empty() {
                        check_path.push('_');
                    }
                    check_path.push_str(check_field_name);

                    // Generate: `if <field>.is_empty() { vec![] } else { vec![`
                    mapping.push_str("if ");
                    mapping.push_str(&check_path);
                    mapping.push_str(".is_empty() { vec![] } else { vec![");
                } else {
                    // No string field to check - fall back to always creating the vec
                    mapping.push_str("vec![");
                }
            }
            _ => {
                mapping.push_str("Some(");
            }
        };

        // Generate the message type and opening brace: `ModulePath::MessageType {`
        mapping.push_str(module_path);
        mapping.push_str(field_type_name);
        mapping.push('{');

        // Recursively generate all field assignments
        for field in &field_type_message.field {
            write_field(
                module_path,
                proto,
                command_field_parts,
                mapping,
                field,
                panic_on_issue,
            )
        }

        // Close the message and vec/option
        match field_type {
            FieldType::VecMessage => {
                if first_string_field.is_some() {
                    // Close: `}] },` (message, vec, conditional block, field separator)
                    mapping.push_str("}] },");
                } else {
                    // Close: `}],` (message, vec, field separator)
                    mapping.push_str("}],");
                }
            }
            _ => {
                // Close: `}),` (message, Some(), field separator)
                mapping.push_str("}),");
            }
        };
    }

    fn write_field(
        module_path: &str,
        proto: &ParsedAndTypechecked,
        command_field_parts: &mut VecDeque<String>,
        mapping: &mut String,
        field: &FieldDescriptorProto,
        panic_on_issue: bool,
    ) {
        let field_type = FieldType::resolve(field, panic_on_issue);
        match field_type {
            FieldType::Map => {}
            _ => {
                let name = field.name();

                if ["type"].contains(&name) {
                    // rust reserved keyword
                    mapping.push_str("r#");
                }

                mapping.push_str(name);
                mapping.push(':');
                command_field_parts.push_back(name.into());
            }
        }

        match field_type {
            FieldType::Primitive | FieldType::VecPrimitive => {
                write_value_from_field(
                    command_field_parts,
                    mapping,
                    field,
                    panic_on_issue,
                );
            }
            FieldType::Message | FieldType::VecMessage => {
                write_value_from_type(
                    module_path,
                    proto,
                    command_field_parts,
                    mapping,
                    field,
                    panic_on_issue,
                );
            }
            FieldType::Map => {}
        }

        match field_type {
            FieldType::Map => {}
            _ => {
                let _ = command_field_parts.pop_back();
            }
        }
    }

    let mut mapping = String::new();

    let input_type =
        proto_reader::helpers::to_unqualified_type(method.input_type());

    let module_path = format!("::proto::{}::", module.to_token_stream());
    mapping.push_str(&module_path);
    mapping.push_str(input_type);
    mapping.push('{');

    let req_message = proto_reader::helpers::find_message(
        proto,
        proto_reader::helpers::to_unqualified_type(method.input_type()),
    )
    .expect("req message");

    let mut command_field_parts = VecDeque::new();
    for field in &req_message.field {
        write_field(
            &module_path,
            proto,
            &mut command_field_parts,
            &mut mapping,
            field,
            panic_on_issue,
        );
    }

    mapping.push_str("};");
    mapping
}
