#!/usr/bin/env auraescript
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
 *                                                                            * * -------------------------------------------------------------------------- * *                                                                            *
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
import * as aurae from "../auraescript/gen/aurae.ts";
import * as cells from "../auraescript/gen/cells.ts";
import * as vms from "../auraescript/gen/vms.ts";

function wait(ms) {
    var start = new Date().getTime();
    var end = start;
    while (end < start + ms) {
        end = new Date().getTime();
    }
}

const client = await aurae.createClient();
const vmService = new vms.VmServiceClient(client);

let vm = await vmService.create(<vms.VmServiceCreateRequest>{
    machine: vms.VirtualMachine.fromPartial({
        id: "ae-sleeper-vm",
        vcpuCount: 2,
        memSizeMb: 1024,
        kernelImgPath: "/var/lib/aurae/vm/kernel/vmlinux.bin",
        kernelArgs: ["console=hvc0", "root=/dev/vda1", "rw"],
        rootDrive: vms.RootDrive.fromPartial({
            imagePath: "/var/lib/aurae/vm/image/disk.raw"
        }),
    })
});
console.log('Created VM:', vm)

// Start and list the VMs
let created = await vmService.start(<vms.VmServiceStartRequest>{ vmId: "ae-sleeper-vm" });
console.log('Started VM:', created)

// Wait 5s for the Vm to be ready
wait(5000);

let machines = await vmService.list(<vms.VmServiceListRequest>{});
console.log('Listed VMs:', machines)

// Stop the VM
await vmService.stop(<vms.VmServiceStopRequest>{ vmId: "ae-sleeper-vm" });

// Wait 5s for the Vm to stop
wait(5000);

// Free the VM
await vmService.free(<vms.VmServiceFreeRequest>{ vmId: "ae-sleeper-vm" });

