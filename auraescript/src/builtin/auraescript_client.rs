use anyhow::Result;
use client::{AuraeConfig, Client};
use deno_core::{Op, OpState, Resource, ResourceId};
use std::{cell::RefCell, rc::Rc};

// `AuraeConfig` `try_default`
#[deno_core::op]
pub(crate) fn as__aurae_config__try_default(
    op_state: &mut OpState,
) -> Result<ResourceId> {
    let config = AuraeConfig::try_default()?;
    let rid = op_state.resource_table.add(AuraeScriptConfig(config));
    Ok(rid)
}

// `AuraeConfig` `from_options`
#[deno_core::op]
pub(crate) fn as__aurae_config__from_options(
    op_state: &mut OpState,
    ca_crt: String,
    client_crt: String,
    client_key: String,
    socket: String,
) -> ResourceId {
    let config =
        AuraeConfig::from_options(ca_crt, client_crt, client_key, socket);
    op_state.resource_table.add(AuraeScriptConfig(config))
}

// `AuraeConfig` `parse_from_file`
#[deno_core::op]
pub(crate) fn as__aurae_config__parse_from_file(
    op_state: &mut OpState,
    path: String,
) -> Result<ResourceId> {
    let config = AuraeConfig::parse_from_toml_file(path)?;
    let rid = op_state.resource_table.add(AuraeScriptConfig(config));
    Ok(rid)
}

// Re export AuraeConfig in auraescript to be able
// to impl Resource on it
pub(crate) struct AuraeScriptConfig(pub AuraeConfig);

impl Resource for AuraeScriptConfig {} // Blank impl

// Create a `Client` with given `AuraeConfig`
#[deno_core::op]
pub(crate) async fn as__client_new(
    op_state: Rc<RefCell<OpState>>,
    config: ResourceId,
) -> Result<ResourceId> {
    let config = {
        let op_state = &op_state.borrow();
        let rt = &op_state.resource_table; // get `ResourceTable` from JsRuntime `OpState`
        rt.get::<AuraeScriptConfig>(config)?.0.clone() // get `Config` from its rid
    };
    let client = Client::new(config).await?;
    let mut op_state = op_state.borrow_mut();
    let rid = op_state.resource_table.add(AuraeScriptClient(client));
    Ok(rid)
}

pub(crate) fn op_decls() -> Vec<::deno_core::OpDecl> {
    vec![
        as__aurae_config__try_default::DECL,
        as__aurae_config__from_options::DECL,
        as__aurae_config__parse_from_file::DECL,
        as__client_new::DECL,
    ]
}

// Re export client in auraescript to be able
// to impl Resource on it
pub(crate) struct AuraeScriptClient(pub Client);

impl Resource for AuraeScriptClient {} // Blank impl
