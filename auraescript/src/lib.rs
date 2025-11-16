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

use deno_ast::{MediaType, ParseParams};
use deno_core::{
    ModuleLoadReferrer, ModuleLoadResponse, ModuleLoader, ModuleSource,
    ModuleSourceCode, ModuleSpecifier, ModuleType, RequestedModuleType,
    ResolutionKind,
    error::{JsError, ModuleLoaderError},
    resolve_import,
    url::Url,
};
use deno_error::JsErrorBox;
use deno_resolver::npm::{DenoInNpmPackageChecker, NpmResolver};
use deno_runtime::{
    BootstrapOptions, FeatureChecker, WorkerLogLevel,
    deno_broadcast_channel::InMemoryBroadcastChannel,
    deno_fs::{FileSystem, RealFs},
    deno_permissions::PermissionsContainer,
    deno_web::BlobStore,
    permissions::RuntimePermissionDescriptorParser,
    worker::{MainWorker, WorkerOptions, WorkerServiceOptions},
};
use std::rc::Rc;
use std::sync::Arc;

mod builtin;
mod cells;
mod cri;
mod discovery;
mod health;
mod observe;
mod vms;

// Load the snapshot of the Deno javascript runtime
static RUNTIME_SNAPSHOT: &[u8] = include_bytes!("../gen/runtime.bin");

deno_core::extension!(auraescript, ops_fn = stdlib);

pub fn init(main_module: Url) -> MainWorker {
    let permission_desc_parser = Arc::new(
        RuntimePermissionDescriptorParser::new(sys_traits::impls::RealSys),
    );
    let permissions = PermissionsContainer::allow_all(permission_desc_parser);

    let fs: Arc<dyn FileSystem> = Arc::new(RealFs);
    let worker_services = WorkerServiceOptions::<
        DenoInNpmPackageChecker,
        NpmResolver<sys_traits::impls::RealSys>,
        sys_traits::impls::RealSys,
    > {
        blob_store: Arc::new(BlobStore::default()),
        broadcast_channel: InMemoryBroadcastChannel::default(),
        deno_rt_native_addon_loader: None,
        feature_checker: Arc::new(FeatureChecker::default()),
        fs,
        module_loader: Rc::new(TypescriptModuleLoader),
        node_services: None,
        npm_process_state_provider: None,
        permissions,
        root_cert_store_provider: None,
        fetch_dns_resolver: deno_runtime::deno_fetch::dns::Resolver::default(),
        shared_array_buffer_store: None,
        compiled_wasm_module_store: None,
        v8_code_cache: None,
        bundle_provider: None,
    };

    let worker_options = WorkerOptions {
        bootstrap: BootstrapOptions {
            args: vec![],
            cpu_count: 1,
            enable_testing_features: false,
            locale: deno_core::v8::icu::get_language_tag(),
            location: None,
            log_level: WorkerLogLevel::Info,
            user_agent: "".to_string(),
            inspect: false,
            ..Default::default()
        },
        extensions: vec![auraescript::init()],
        startup_snapshot: Some(RUNTIME_SNAPSHOT),
        format_js_error_fn: Some(Arc::new(|error: &JsError| {
            deno_runtime::fmt_errors::format_js_error(error)
        })),
        ..Default::default()
    };

    MainWorker::bootstrap_from_options(
        &main_module,
        worker_services,
        worker_options,
    )
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
    ops.extend(vms::op_decls());
    ops
}

// From: https://github.com/denoland/deno/blob/main/core/examples/ts_module_loader.rs
struct TypescriptModuleLoader;

impl ModuleLoader for TypescriptModuleLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _is_main: ResolutionKind,
    ) -> Result<ModuleSpecifier, ModuleLoaderError> {
        resolve_import(specifier, referrer)
            .map_err(|err| JsErrorBox::generic(err.to_string()))
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<&ModuleLoadReferrer>,
        _is_dyn_import: bool,
        _requested_module_type: RequestedModuleType,
    ) -> ModuleLoadResponse {
        fn load_specifier(
            module_specifier: &ModuleSpecifier,
        ) -> Result<ModuleSource, ModuleLoaderError> {
            let path = module_specifier.to_file_path().map_err(|_| {
                JsErrorBox::generic("Only file: URLs are supported.")
            })?;

            let media_type = MediaType::from_path(&path);
            let (module_type, should_transpile) = match media_type {
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
                _ => {
                    return Err(JsErrorBox::generic(format!(
                        "Unknown extension {:?}",
                        path.extension()
                    )));
                }
            };

            let code = std::fs::read_to_string(&path)
                .map_err(|err| JsErrorBox::generic(err.to_string()))?;
            let code = if should_transpile {
                let parsed = deno_ast::parse_module(ParseParams {
                    specifier: module_specifier.clone(),
                    text: code.into(),
                    media_type,
                    capture_tokens: false,
                    scope_analysis: false,
                    maybe_syntax: None,
                })
                .map_err(|err| JsErrorBox::generic(err.to_string()))?;
                parsed
                    .transpile(
                        &deno_ast::TranspileOptions::default(),
                        &deno_ast::TranspileModuleOptions::default(),
                        &deno_ast::EmitOptions::default(),
                    )
                    .map_err(|err| JsErrorBox::generic(err.to_string()))?
                    .into_source()
                    .text
            } else {
                code
            };
            Ok(ModuleSource::new_with_redirect(
                module_type,
                ModuleSourceCode::String(code.into()),
                module_specifier,
                module_specifier,
                None,
            ))
        }

        ModuleLoadResponse::Sync(load_specifier(module_specifier))
    }
}
