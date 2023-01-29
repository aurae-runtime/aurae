/* -------------------------------------------------------------------------- *\
 *             Apache 2.0 License Copyright © 2022-2023 The Aurae Authors          *
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

use aurae_proto::cri::PodSandboxConfig;
use oci_spec::runtime::{Spec, SpecBuilder};
use oci_spec::OciSpecError;

pub struct AuraeOCIBuilder {
    spec_builder: SpecBuilder,
}

impl AuraeOCIBuilder {
    pub fn new() -> AuraeOCIBuilder {
        AuraeOCIBuilder {
            // TODO Port config.json to this builder
            spec_builder: SpecBuilder::default().version("1.0.2-dev"),
        }
    }
    pub fn overload_pod_sandbox_config(
        self,
        _config: PodSandboxConfig,
    ) -> AuraeOCIBuilder {
        // TODO Map the Linux security context, mounts, ports, etc to the OCI spec
        // Appends the current pod config to the SpecBuilder
        self
    }
    pub fn build(self) -> Result<Spec, OciSpecError> {
        self.spec_builder.build()
    }
}
