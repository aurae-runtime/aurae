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
#![allow(dead_code)]

use libcontainer::container::Container;

#[derive(Debug, Clone, Default)]
pub struct Sandbox {
    /// The unique name of the Pod sandbox at runtime.
    ///
    /// Note: This is the name of the "Pod" that can typically be associated
    /// back the AURAE_RUNTIME_DIR value (which is typically "/var/run/aurae").
    ///
    /// Note: This also is a copy of the value that is used in the cache hashmap
    /// to access the Pod sandbox in the internal cache mechanism.
    name: String,

    /// Init containers are the "preliminary" container that is used to begin
    /// the isolation process in a sandbox.
    ///
    /// The init container will most often be a spawned "auraed" instance
    /// running in a new namespace isolation zone that is unshared from the
    /// host namespaces.
    pub(crate) init: Container,

    /// Tenants are the arbitrary workloads running alongside the init
    /// containers in an Aurae pod.
    ///
    /// These are usually things like an OCI compatible container image such
    /// as "nginx" or "busybox".
    ///
    /// In the case of large enterprise workload management, these specifically
    /// are "your app".
    pub(crate) tenants: Vec<Container>,
}

pub struct SandboxBuilder {
    name: String,
    init: Container,
}

impl SandboxBuilder {
    // TODO: Consider embedding the ContainerBuilder directly into this SandboxBuilder. For now just require a started init container.
    pub fn new(name: String, init: Container) -> SandboxBuilder {
        SandboxBuilder { name, init }
    }
    /// The SandboxBuilder will require that the libcontainer::Container be built before
    /// we can build the Sandbox.
    pub fn build(self) -> Sandbox {
        Sandbox { name: self.name, init: self.init, tenants: vec![] }
    }
}