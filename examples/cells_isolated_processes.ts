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
const cellName = "ae-1";

// [ Allocate Shared NS ]
let s_allocated = await cellService.allocate(<cells.CellServiceAllocateRequest>{
    cell: cells.Cell.fromPartial({
        cpu: cells.CpuController.fromPartial({
            weight: 2
        }),
        name: cellName,
        isolateNetwork: false,
        isolateProcess: true,
    })
});
aurae.print(s_allocated)

// [ Start ]
let s_started = await cellService.start(<cells.CellServiceStartRequest>{
    cellName: cellName,
    executable: cells.Executable.fromPartial({
        command: "ls /proc",
        description: "List processes",
        name: "ps-aux"
    })
})
aurae.print(s_started)

let host_started = await cellService.start(<cells.CellServiceStartRequest>{
    cellName: cellName,
    executable: cells.Executable.fromPartial({
        command: "hostname",
        description: "Show hostname",
        name: "show-hostname"
    })
})
aurae.print(host_started)

// [ Free ]
await cellService.free(<cells.CellServiceFreeRequest>{
    cellName: cellName
});
