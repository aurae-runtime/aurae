#!/usr/bin/env auraescript
/* -------------------------------------------------------------------------- *\
 *             Apache 2.0 License Copyright © 2022 The Aurae Authors          *
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
import * as runtime from "../auraescript/gen/runtime.ts";

let cells = new runtime.CellServiceClient();

const cellName = "ae-1";
// [ Allocate Shared NS ]
let s_allocated = await cells.allocate(<runtime.AllocateCellRequest>{
    cell: runtime.Cell.fromPartial({
        cpuShares: 2,
        name: cellName,
        // nsShareMount: true,
        // nsShareUts: true,
        // nsShareIpc: true,
        // nsSharePid: true,
        // nsShareNet: true,
        // nsShareCgroup: true,
    })
});
helpers.print(s_allocated)

// [ Start ]
let s_started = await cells.start(<runtime.StartExecutableRequest>{
    cellName: cellName,
    executable: runtime.Executable.fromPartial({
        command: "ls /proc", // Note: you must use the full path now for namespaces!
        description: "List processes",
        name: "ps-aux"
    })
})
helpers.print(s_started)

// [ Free ]
await cells.free(<runtime.FreeCellRequest>{
    cellName: cellName
});
