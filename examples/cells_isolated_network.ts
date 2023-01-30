#!/usr/bin/env auraescript
/* -------------------------------------------------------------------------- *\
 *               Apache 2.0 License Copyright The Aurae Authors               *
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

import * as helpers from "../auraescript/gen/helpers.ts";
import * as runtime from "../auraescript/gen/cells.ts";

let cells = new runtime.CellServiceClient();
const cellName = "ae-net-1";

// [ Allocate Shared NS ]
let net_allocated = await cells.allocate(<runtime.CellServiceAllocateRequest>{
    cell: runtime.Cell.fromPartial({
        cpu: runtime.CpuController.fromPartial({
            weight: 2
        }),
        name: cellName,
        isolateNetwork: true,
        isolateProcess: false,
    })
});
helpers.print(net_allocated)

// [ Start ]
let net_started = await cells.start(<runtime.CellServiceStartRequest>{
    cellName: cellName,
    executable: runtime.Executable.fromPartial({
        command: "ifconfig && ip a && route",
        description: "Show network info",
        name: "net-info"
    })
})
helpers.print(net_started)

// [ Free ]
await cells.free(<runtime.CellServiceFreeRequest>{
    cellName: cellName
});
