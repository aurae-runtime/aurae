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
let allocated = await cells.allocate(<runtime.AllocateCellRequest>{
    cell: runtime.Cell.fromPartial({
        cpuQuota: 400 * (10 ** 3), // 0.4 seconds in microseconds
        cpuShares: 2, // Percentage of CPUs
        name: "echo-cell",
    })
});
//helpers.print(allocated)

// [ Start ]
let started_out = await cells.start(<runtime.StartExecutableRequest>{
    cellName: "echo-cell",
    executable: runtime.Executable.fromPartial({
        command: "/usr/bin/echo 'hello world'",
        description: "outputs a message to stdout",
        name: "echo-stdout"
    })
})
//helpers.print(started_out)

// [ Start ]
let started_err = await cells.start(<runtime.StartExecutableRequest>{
    cellName: "echo-cell",
    executable: runtime.Executable.fromPartial({
        command: "/usr/bin/echo 'hello world' 1>&2",
        description: "outputs a message to stderr",
        name: "echo-stderr"
    })
})

// [ Stop ]
let stopped_out = await cells.stop(<runtime.StopExecutableRequest>{
    cellName: "echo-cell",
    executableName: "echo-stdout",
})
// [ Stop ]
let stopped_err = await cells.stop(<runtime.StopExecutableRequest>{
    cellName: "echo-cell",
    executableName: "echo-stderr",
})
//helpers.print(stopped)

// [ Free ]
let freed = await cells.free(<runtime.FreeCellRequest>{
    cellName: "echo-cell"
});
//helpers.print(freed)
