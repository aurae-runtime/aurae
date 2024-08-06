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
/* -------------------------------------------------------------------------- *\
*        Apache 2.0 License Copyright © 2022-2023 The Aurae Authors          *
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
//! # AuraeScript
//!
//! AuraeScript is a turing complete language for platform teams built on [Deno](https://deno.land).
//!
//! AuraeScript is a lightweight client that wraps the [Aurae Standard Library](https://aurae.io/stdlib/).
//!
//! AuraeScript is a quick way to access the core Aurae APIs and follows normal UNIX parlance. AuraeScript should feel simple and intuitive for any Go, C, Python, or Rust programmer.
//!
//! ### Architecture
//!
//! AuraeScript follows a similar client paradigm to Kubernetes `kubectl` command. However, unlike Kubernetes this is not a command line tool like `kubectl`. AuraeScript is a fully supported programing language complete with a systems standard library. The Aurae runtime projects supports many clients, and the easiest client to get started building with is AuraeScript.
//!
//! Download the static binary directly to your system, and you can begin writing AuraeScript programs directly against a running Aurae server.

// Lint groups: https://doc.rust-lang.org/rustc/lints/groups.html
#![warn(future_incompatible, nonstandard_style, unused)]
#![warn(
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    unconditional_recursion,
    unused_comparisons,
    while_true
)]
#![warn(missing_debug_implementations,
    // TODO: missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_qualifications,
    unused_results
)]
#![warn(clippy::unwrap_used)]
// TODO: need to figure out how to get tonic to allow this without allowing for whole crate
#![allow(unused_qualifications)]

use anyhow::{anyhow, bail, Error};
use deno_ast::{EmitOptions, MediaType, ParseParams, SourceMapOption};
use deno_core::{
    self, error::AnyError, resolve_import, url::Url, JsRuntime,
    ModuleLoadResponse, ModuleLoader, ModuleSource, ModuleSourceCode,
    ModuleSpecifier, ModuleType, RequestedModuleType, ResolutionKind,
    RuntimeOptions, SourceMapGetter,
};
use std::{cell::RefCell, collections::HashMap, future::Future, rc::Rc};

mod builtin;
mod cells;
mod cri;
mod discovery;
mod health;
mod observe;

fn get_error_class_name(e: &AnyError) -> &'static str {
    deno_runtime::errors::get_error_class_name(e).unwrap_or("Error")
}

deno_core::extension!(auraescript, ops_fn = stdlib);

pub fn runtime(main_module: Url) -> impl Future<Output = Result<(), Error>> {
    let source_map_store =
        SourceMapStore(Rc::new(RefCell::new(HashMap::new())));
    let mut runtime = JsRuntime::new(RuntimeOptions {
        module_loader: Some(Rc::new(TypescriptModuleLoader {
            source_maps: source_map_store.clone(),
        })),
        source_map_getter: Some(Rc::new(source_map_store)),
        extensions: vec![auraescript::init_ops()],
        get_error_class_fn: Some(&get_error_class_name),
        ..Default::default()
    });

    async move {
        let mod_id = runtime.load_main_es_module(&main_module).await?;
        let result = runtime.mod_evaluate(mod_id);
        runtime.run_event_loop(Default::default()).await?;
        result.await
    }
}

/// Standard Library Autogeneration Code
///
/// To add an auto generated package to AuraeScript it MUST
/// be defined in this function.
///
/// Add a similar line to the function for each newly implemented
/// service.
///
/// ops.extend(my_package::op_decls());
///
fn stdlib() -> Vec<deno_core::OpDecl> {
    let mut ops = vec![];
    ops.extend(builtin::auraescript_client::op_decls());
    ops.extend(cells::op_decls());
    ops.extend(cri::op_decls());
    ops.extend(discovery::op_decls());
    ops.extend(health::op_decls());
    ops.extend(observe::op_decls());
    ops
}

// From: https://github.com/denoland/deno/blob/main/core/examples/ts_module_loader.rs
#[derive(Clone)]
struct SourceMapStore(Rc<RefCell<HashMap<String, Vec<u8>>>>);

impl SourceMapGetter for SourceMapStore {
    fn get_source_map(&self, specifier: &str) -> Option<Vec<u8>> {
        self.0.borrow().get(specifier).cloned()
    }

    fn get_source_line(
        &self,
        _file_name: &str,
        _line_number: usize,
    ) -> Option<String> {
        None
    }
}

struct TypescriptModuleLoader {
    source_maps: SourceMapStore,
}

impl ModuleLoader for TypescriptModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _is_main: ResolutionKind,
    ) -> Result<ModuleSpecifier, Error> {
        Ok(resolve_import(specifier, referrer)?)
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<&ModuleSpecifier>,
        _is_dyn_import: bool,
        _requested_module_type: RequestedModuleType,
    ) -> ModuleLoadResponse {
        fn load(
            source_maps: SourceMapStore,
            module_specifier: &ModuleSpecifier,
        ) -> Result<ModuleSource, Error> {
            let path = module_specifier
                .to_file_path()
                .map_err(|_| anyhow!("Only file:// URLs are supported."))?;

            let media_type = MediaType::from_path(&path);
            let (module_type, should_transpile) =
                match MediaType::from_path(&path) {
                    MediaType::JavaScript | MediaType::Mjs | MediaType::Cjs => {
                        (ModuleType::JavaScript, false)
                    }
                    MediaType::Jsx => (ModuleType::JavaScript, true),
                    MediaType::TypeScript
                    | MediaType::Mts
                    | MediaType::Cts
                    | MediaType::Dts
                    | MediaType::Dmts
                    | MediaType::Dcts
                    | MediaType::Tsx => (ModuleType::JavaScript, true),
                    MediaType::Json => (ModuleType::Json, false),
                    _ => bail!("Unknown extension {:?}", path.extension()),
                };

            let code = std::fs::read_to_string(&path)?;
            let code = if should_transpile {
                let parsed = deno_ast::parse_module(ParseParams {
                    specifier: module_specifier.clone(),
                    text: code.into(),
                    media_type,
                    capture_tokens: false,
                    scope_analysis: false,
                    maybe_syntax: None,
                })?;
                let source = parsed
                    .transpile(
                        &Default::default(),
                        &EmitOptions {
                            source_map: SourceMapOption::Separate,
                            inline_sources: true,
                            ..Default::default()
                        },
                    )?
                    .into_source();

                if let Some(map) = source.source_map {
                    let _ = source_maps
                        .0
                        .borrow_mut()
                        .insert(module_specifier.to_string(), map);
                }
                String::from_utf8(source.source)?
            } else {
                code
            };
            Ok(ModuleSource::new(
                module_type,
                ModuleSourceCode::String(code.into()),
                module_specifier,
                None,
            ))
        }
        ModuleLoadResponse::Sync(load(
            self.source_maps.clone(),
            module_specifier,
        ))
    }
}