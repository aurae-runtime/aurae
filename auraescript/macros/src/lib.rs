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
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

mod get_set;

/// Outputs the macro content during a render.
#[proc_macro_derive(Output)]
pub fn output(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let ident = &ast.ident;

    let expanded = quote! {
        impl #ident {
            /// Output as symmetrical AuraeScript code.
            pub fn raw(&mut self) {
                println!("{:?}", self);
            }

            /// Output as valid JSON.
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
