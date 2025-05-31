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

use proc_macro2::Span;
use protobuf::descriptor::field_descriptor_proto::Type;
use protobuf::descriptor::DescriptorProto;
use protobuf_parse::ParsedAndTypechecked;
use syn::Ident;

pub fn to_unqualified_type(t: &str) -> &str {
    t.split('.').next_back().expect("to_unqualified_type")
}

// Better way?
pub fn to_rust_type(field_type: Type, span: Span) -> Ident {
    match field_type {
        Type::TYPE_DOUBLE => Ident::new("f64", span),
        Type::TYPE_FLOAT => Ident::new("f32", span),
        Type::TYPE_INT64 => Ident::new("i64", span),
        Type::TYPE_UINT64 => Ident::new("u64", span),
        Type::TYPE_INT32 => Ident::new("i32", span),
        Type::TYPE_FIXED64 => Ident::new("u64", span),
        Type::TYPE_FIXED32 => Ident::new("u32", span),
        Type::TYPE_BOOL => Ident::new("bool", span),
        Type::TYPE_STRING => Ident::new("String", span),
        // Type::TYPE_GROUP => todo!(),
        // Type::TYPE_MESSAGE => todo!(),
        Type::TYPE_BYTES => Ident::new("Vec<u8>", span),
        Type::TYPE_UINT32 => Ident::new("u32", span),
        Type::TYPE_ENUM => Ident::new("i32", span),
        Type::TYPE_SFIXED32 => Ident::new("i32", span),
        Type::TYPE_SFIXED64 => Ident::new("i64", span),
        Type::TYPE_SINT32 => Ident::new("i32", span),
        Type::TYPE_SINT64 => Ident::new("i64", span),
        other => panic!("type not supported: {other:?}"),
    }
}

pub fn find_message<'a>(
    proto: &'a ParsedAndTypechecked,
    name: &str,
) -> Option<&'a DescriptorProto> {
    proto
        .file_descriptors
        .iter()
        .flat_map(|f| &f.message_type)
        .find(|m| matches!(m.name(), n if name == n))
}
