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
import * as aurae from "../auraescript/gen/aurae.ts";
import * as cells from "../auraescript/gen/cells.ts";

let client = await aurae.createClient();
let cellService = new cells.CellServiceClient(client);
const cellName = "ae-net-1";

// [ Allocate Shared NS ]
let net_allocated = await cellService.allocate(<cells.CellServiceAllocateRequest>{
    cell: cells.Cell.fromPartial({
        cpu: cells.CpuController.fromPartial({
            weight: 2
        }),
        name: cellName,
        isolateNetwork: true,
        isolateProcess: false,
    })
});
aurae.print(net_allocated)

// [ Start ]
let net_started = await cellService.start(<cells.CellServiceStartRequest>{
    cellName: cellName,
    executable: cells.Executable.fromPartial({
        command: "ifconfig && ip a && route",
        description: "Show network info",
        name: "net-info"
    })
})
aurae.print(net_started)

// [ Free ]
await cellService.free(<cells.CellServiceFreeRequest>{
    cellName: cellName
});
