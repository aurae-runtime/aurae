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

// [ Allocate ]
let allocated = await cells.allocate(<runtime.CellServiceAllocateRequest>{
    cell: runtime.Cell.fromPartial({
        cpuQuota: 400 * (10 ** 3), // 0.4 seconds in microseconds
        cpuShares: 2, // Percentage of CPUs
        name: "sleeper-cell",
    })
});
helpers.print(allocated)

// [ Start ]
let started = await cells.start(<runtime.CellServiceStartRequest>{
    cellName: "sleeper-cell",
    executable: runtime.Executable.fromPartial({
        command: "/usr/bin/sleep 42",
        description: "Sleep for 42 seconds",
        name: "sleep-42"
    })
})
helpers.print(started)

// [ Stop ]
let stopped = await cells.stop(<runtime.CellServiceStopRequest>{
    cellName: "sleeper-cell",
    executableName: "sleep-42",
})
helpers.print(stopped)

// [ Free ]
let freed = await cells.free(<runtime.CellServiceFreeRequest>{
    cellName: "sleeper-cell"
});
helpers.print(freed)