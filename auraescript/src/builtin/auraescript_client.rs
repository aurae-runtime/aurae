use anyhow::Result;
use client::Client;
use deno_core::{OpState, Resource, ResourceId};
use std::{cell::RefCell, rc::Rc};

#[deno_core::op]
pub(crate) async fn auraescript_client(
    opstate: Rc<RefCell<OpState>>,
    ca_crt: String,
    client_crt: String,
    client_key: String,
    socket: String,
) -> Result<ResourceId> {
    let client =
        Client::from_options(ca_crt, client_crt, client_key, socket).await?;
    let mut opstate = opstate.borrow_mut();
    let rid = opstate.resource_table.add(AuraeScriptClient(client));
    Ok(rid)
}

pub(crate) fn op_decls() -> Vec<::deno_core::OpDecl> {
    vec![auraescript_client::decl()]
}

pub(crate) struct AuraeScriptClient(pub Client);

impl Resource for AuraeScriptClient {} // Blank impl
