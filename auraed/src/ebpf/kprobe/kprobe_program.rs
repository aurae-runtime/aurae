# ---------------------------------------------------------------------------- #
#                +--------------------------------------------+                #
#                |   █████╗ ██╗   ██╗██████╗  █████╗ ███████╗ |                #
#                |  ██╔══██╗██║   ██║██╔══██╗██╔══██╗██╔════╝ |                #
#                |  ███████║██║   ██║██████╔╝███████║█████╗   |                #
#                |  ██╔══██║██║   ██║██╔══██╗██╔══██║██╔══╝   |                #
#                |  ██║  ██║╚██████╔╝██║  ██║██║  ██║███████╗ |                #
#                |  ╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚══════╝ |                #
#                +--------------------------------------------+                #
#                                                                              #
#                         Distributed Systems Runtime                          #
# ---------------------------------------------------------------------------- #
# Copyright 2022 - 2024, the aurae contributors
# SPDX-License-Identifier: Apache-2.0

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
use aya::programs::{KProbe, ProgramError};
use aya::Bpf;
use tracing::{trace, warn};

pub trait KProbeProgram<T: Clone + Send + 'static> {
    const PROGRAM_NAME: &'static str;
    const FUNCTION_NAME: &'static str;
    const PERF_BUFFER: &'static str;

    fn load_and_attach(bpf: &mut Bpf) -> Result<(), anyhow::Error> {
        trace!("Loading eBPF program: {}", Self::PROGRAM_NAME);

        // Load the eBPF TracePoint program
        let program: &mut KProbe = bpf
            .program_mut(Self::PROGRAM_NAME)
            .ok_or_else(|| anyhow::anyhow!("failed to get eBPF program"))?
            .try_into()?;

        // Load the program
        match program.load() {
            Ok(_) => Ok(()),
            Err(ProgramError::AlreadyLoaded) => {
                warn!("Already loaded eBPF program {}", Self::PROGRAM_NAME);
                Ok(())
            }
            other => other,
        }?;

        // Attach to kernel trace event
        match program.attach(Self::FUNCTION_NAME, 0) {
            Ok(_) => Ok(()),
            Err(ProgramError::AlreadyAttached) => {
                warn!("Already attached eBPF program {}", Self::PROGRAM_NAME);
                Ok(())
            }
            Err(e) => Err(e),
        }?;

        Ok(())
    }
}
