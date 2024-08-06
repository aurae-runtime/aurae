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

use crate::AURAED_RUNTIME;
use aya::{Bpf, BpfError};
use tracing::trace;

pub trait BpfFile {
    const OBJ_NAME: &'static str;

    fn load() -> Result<Bpf, BpfError> {
        trace!("Loading eBPF file: {}", Self::OBJ_NAME);

        Bpf::load_file(format!(
            "{}/ebpf/{}",
            AURAED_RUNTIME
                .get()
                .expect("runtime")
                .library_dir
                .to_string_lossy(),
            Self::OBJ_NAME
        ))
    }
}