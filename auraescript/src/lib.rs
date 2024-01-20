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
use deno_ast::{MediaType, ParseParams, SourceTextInfo};
use deno_runtime::{
    deno_core::{
        self, error::AnyError, futures::FutureExt, resolve_import, url::Url,
        FastString, ModuleLoader, ModuleSource, ModuleSourceCode,
        ModuleSourceFuture, ModuleSpecifier, ModuleType, ResolutionKind,
        Snapshot,
    },
    permissions::PermissionsContainer,
    worker::{MainWorker, WorkerOptions},
    BootstrapOptions, WorkerLogLevel,
};

use std::pin::Pin;
use std::rc::Rc;

mod builtin;
mod cells;
mod cri;
mod discovery;
mod health;
mod observe;

// Load the snapshot of the Deno javascript runtime
static RUNTIME_SNAPSHOT: &[u8] = include_bytes!("../gen/runtime.bin");

fn get_error_class_name(e: &AnyError) -> &'static str {
    deno_runtime::errors::get_error_class_name(e).unwrap_or("Error")
}

deno_core::extension!(auraescript, ops_fn = stdlib);

pub fn init(main_module: Url) -> MainWorker {
    MainWorker::bootstrap_from_options(
        main_module,
        PermissionsContainer::allow_all(),
        WorkerOptions {
            extensions: vec![auraescript::init_ops()],
            module_loader: Rc::new(TypescriptModuleLoader),
            get_error_class_fn: Some(&get_error_class_name),
            startup_snapshot: Some(Snapshot::Static(RUNTIME_SNAPSHOT)),
            bootstrap: BootstrapOptions {
                args: vec![],
                cpu_count: 1,
                locale: deno_core::v8::icu::get_language_tag(),
                log_level: WorkerLogLevel::Debug,
                no_color: false,
                is_tty: false,
                unstable: true,
                user_agent: "".to_string(),
                inspect: false,
                ..Default::default()
            },
            ..Default::default()
        },
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
    ) -> Result<ModuleSpecifier, Error> {
        Ok(resolve_import(specifier, referrer)?)
    }

    fn load(
        &self,
        module_specifier: &ModuleSpecifier,
        _maybe_referrer: Option<&ModuleSpecifier>,
        _is_dyn_import: bool,
    ) -> Pin<Box<ModuleSourceFuture>> {
        let module_specifier = module_specifier.clone();
        async move {
            let path = module_specifier
                .to_file_path()
                .map_err(|_| anyhow!("Only file: URLs are supported."))?;

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
                    specifier: module_specifier.to_string(),
                    text_info: SourceTextInfo::from_string(code),
                    media_type,
                    capture_tokens: false,
                    scope_analysis: false,
                    maybe_syntax: None,
                })?;
                parsed.transpile(&Default::default())?.text
            } else {
                code
            };
            let module = ModuleSource::new_with_redirect(
                module_type,
                ModuleSourceCode::String(FastString::Owned(code.into())),
                &module_specifier,
                &module_specifier,
            );
            Ok(module)
        }
        .boxed_local()
    }
}
