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

use anyhow::Context;
use aya::programs::{ProgramError, TracePoint};
use aya::Bpf;
use tracing::{trace, warn};

pub trait TracepointProgram<T: Clone + Send + 'static> {
    const PROGRAM_NAME: &'static str;
    const CATEGORY: &'static str;
    const EVENT: &'static str;
    const PERF_BUFFER: &'static str;

    fn load_and_attach(bpf: &mut Bpf) -> Result<(), anyhow::Error> {
        trace!("Loading eBPF program: {}", Self::PROGRAM_NAME);

        // Load the eBPF TracePoint program
        let program: &mut TracePoint = bpf
            .program_mut(Self::PROGRAM_NAME)
            .context("failed to get eBPF program")?
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
        match program.attach(Self::CATEGORY, Self::EVENT) {
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
