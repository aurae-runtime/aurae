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
use oci_spec::runtime::{MountBuilder, RootBuilder, Spec, SpecBuilder};
use oci_spec::OciSpecError;

pub struct AuraeOCIBuilder {
    spec_builder: SpecBuilder,
}

impl AuraeOCIBuilder {
    pub fn new() -> AuraeOCIBuilder {
        AuraeOCIBuilder {
            // TODO Port config.json to this builder
            spec_builder: SpecBuilder::default()
                .version("1.0.2-dev")
                .root(
                    RootBuilder::default()
                        .path("rootfs")
                        .readonly(false)
                        .build()
                        .expect("default oci: root"),
                )
                .mounts(vec![
                    MountBuilder::default()
                        .destination("/proc")
                        .typ("proc")
                        .source("proc")
                        .build()
                        .expect("default oci: mount /proc"),
                    MountBuilder::default()
                        .destination("/dev")
                        .typ("devtmpfs")
                        .source("devtmpfs")
                        .options(vec![
                            "nosuid".to_string(),
                            "strictatime".to_string(),
                            "mode=755".to_string(),
                            "size=65536k".to_string(),
                        ])
                        .build()
                        .expect("default oci: mount /dev"),
                    MountBuilder::default()
                        .destination("/dev/pts")
                        .typ("devpts")
                        .source("devpts")
                        .options(vec![
                            "nosuid".to_string(),
                            "noexec".to_string(),
                            "newinstance".to_string(),
                            "ptmxmode=0666".to_string(),
                            "mode=0620".to_string(),
                            "gid=5".to_string(),
                        ])
                        .build()
                        .expect("default oci: mount /dev/pts"),
                    MountBuilder::default()
                        .destination("/dev/shm")
                        .typ("tmpfs")
                        .source("shm")
                        .options(vec![
                            "nosuid".to_string(),
                            "noexec".to_string(),
                            "nodev".to_string(),
                            "mode=1777".to_string(),
                            "size=65536k".to_string(),
                        ])
                        .build()
                        .expect("default oci: mount /dev/shm"),
                    MountBuilder::default()
                        .destination("/dev/mqueue")
                        .typ("mqueue")
                        .source("mqueue")
                        .options(vec![
                            "nosuid".to_string(),
                            "noexec".to_string(),
                            "nodev".to_string(),
                        ])
                        .build()
                        .expect("default oci: mount /dev/shm"),
                    MountBuilder::default()
                        .destination("/sys")
                        .typ("sysfs")
                        .source("sysfs")
                        .options(vec![
                            "nosuid".to_string(),
                            "noexec".to_string(),
                            "nodev".to_string(),
                            "ro".to_string(),
                        ])
                        .build()
                        .expect("default oci: mount /sys"),
                    MountBuilder::default()
                        .destination("/sys/fs/cgroup")
                        .typ("cgroup")
                        .source("cgroup")
                        .options(vec![
                            "nosuid".to_string(),
                            "noexec".to_string(),
                            "nodev".to_string(),
                            "relatime".to_string(),
                            "ro".to_string(),
                        ])
                        .build()
                        .expect("default oci: mount /sys/fs/cgroup"),
                ]),
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

//   "process": {
//     "terminal": false,
//     "user": {
//       "uid": 0,
//       "gid": 0
//     },
//     "args": [
//       "init"
//     ],
//     "env": [
//       "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
//       "TERM=xterm"
//     ],
//     "cwd": "/",
//     "capabilities": {
//       "bounding": [
//         "CAP_AUDIT_WRITE",
//         "CAP_NET_BIND_SERVICE",
//         "CAP_KILL"
//       ],
//       "effective": [
//         "CAP_AUDIT_WRITE",
//         "CAP_NET_BIND_SERVICE",
//         "CAP_KILL"
//       ],
//       "inheritable": [
//         "CAP_AUDIT_WRITE",
//         "CAP_NET_BIND_SERVICE",
//         "CAP_KILL"
//       ],
//       "permitted": [
//         "CAP_AUDIT_WRITE",
//         "CAP_NET_BIND_SERVICE",
//         "CAP_KILL"
//       ],
//       "ambient": [
//         "CAP_AUDIT_WRITE",
//         "CAP_NET_BIND_SERVICE",
//         "CAP_KILL"
//       ]
//     },
//     "rlimits": [
//       {
//         "type": "RLIMIT_NOFILE",
//         "hard": 1024,
//         "soft": 1024
//       }
//     ],
//     "noNewPrivileges": true
//   },
//   "hostname": "aurae",
//   "annotations": {},
//   "linux": {
//     "resources": {
//       "devices": [
//         {
//           "allow": false,
//           "type": null,
//           "major": null,
//           "minor": null,
//           "access": "rwm"
//         }
//       ]
//     },
//     "namespaces": [
//       {
//         "type": "pid"
//       },
//       {
//         "type": "network"
//       },
//       {
//         "type": "ipc"
//       },
//       {
//         "type": "uts"
//       },
//       {
//         "type": "mount"
//       }
//     ],
//     "maskedPaths": [
//       "/proc/acpi",
//       "/proc/asound",
//       "/proc/kcore",
//       "/proc/keys",
//       "/proc/latency_stats",
//       "/proc/timer_list",
//       "/sys/firmware",
//       "/proc/scsi"
//     ],
//     "readonlyPaths": [
//       "/proc/bus",
//       "/proc/fs",
//       "/proc/irq",
//       "/proc/sys",
//       "/proc/sysrq-trigger"
//     ]
//   }
// }
