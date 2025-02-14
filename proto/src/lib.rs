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

//! Generated Protobuf definitions for the Aurae Standard Library

#![allow(clippy::derive_partial_eq_without_eq)]
#![allow(clippy::empty_docs)]
#![allow(clippy::match_single_binding)]
#![allow(clippy::doc_lazy_continuation)]
#![allow(clippy::needless_lifetimes)]

pub mod cells {
    include!("../gen/aurae.cells.v0.rs");
}

pub mod discovery {
    include!("../gen/aurae.discovery.v0.rs");
}

pub mod grpc {
    pub mod health {
        include!("../gen/grpc.health.v1.rs");
    }
}

pub mod cri {
    include!("../gen/runtime.v1.rs");
}

pub mod observe {
    include!("../gen/aurae.observe.v0.rs");
}

pub mod vms {
    include!("../gen/aurae.vms.v0.rs");
}
