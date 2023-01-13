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
let cellName = "ae-echo-cell";

// [ Allocate ]
let allocated = await cells.allocate(<runtime.CellServiceAllocateRequest>{
    cell: runtime.Cell.fromPartial({
        cpu: runtime.CpuController.fromPartial({
            weight: 2, // Percentage of CPUs
            limit: 400 * (10 ** 3), // 0.4 seconds in microseconds
        }),
        name: cellName,
    })
});
//helpers.print(allocated)

// [ Start ]
let started_out = await cells.start(<runtime.CellServiceStartRequest>{
    cellName,
    executable: runtime.Executable.fromPartial({
        command: "/usr/bin/echo 'hello world'",
        description: "outputs a message to stdout",
        name: "echo-stdout"
    })
})
//helpers.print(started_out)

// [ Start ]
let started_err = await cells.start(<runtime.CellServiceStartRequest>{
    cellName,
    executable: runtime.Executable.fromPartial({
        command: "/usr/bin/echo 'hello world' 1>&2",
        description: "outputs a message to stderr",
        name: "echo-stderr"
    })
})

// [ Stop ]
let stopped_out = await cells.stop(<runtime.CellServiceStopRequest>{
    cellName,
    executableName: "echo-stdout",
})
// [ Stop ]
let stopped_err = await cells.stop(<runtime.CellServiceStopRequest>{
    cellName,
    executableName: "echo-stderr",
})
//helpers.print(stopped)

// [ Free ]
let freed = await cells.free(<runtime.CellServiceFreeRequest>{
    cellName
});
//helpers.print(freed)
