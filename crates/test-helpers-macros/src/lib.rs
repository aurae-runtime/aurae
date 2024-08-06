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
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn shared_runtime_test(
    _attr: TokenStream,
    input: TokenStream,
) -> TokenStream {
    // Because this attribute goes on a test, we know there are no inputs to the function.

    let ast = parse_macro_input!(input as ItemFn);

    let ItemFn { attrs, vis, sig, block } = ast;
    let mut sig = sig;
    sig.asyncness = None;
    let sig = sig;

    let stmts = &block.stmts;

    let expanded = quote! {
        #[test] #(#attrs)* #vis #sig {
            async fn action() {
                #(#stmts)*
            }

            common::test(action())
        }
    };

    expanded.into()
}