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
import * as helpers from "../auraescript/gen/helpers.ts";
import * as runtime from "../auraescript/gen/cells.ts";

let cells = new runtime.CellServiceClient();
let cellName = "ae-sleeper-cell";

// [ Allocate ]
let allocated = await cells.allocate(<runtime.CellServiceAllocateRequest>{
    cell: runtime.Cell.fromPartial({
        cpu: runtime.CpuController.fromPartial({
            weight: 2, // Percentage of CPUs
            max: 300000, // 30% of the CPU
        }),
        name: cellName,
    })
});
helpers.print(allocated)


// [ Start ]
let started = await cells.start(<runtime.CellServiceStartRequest>{
    cellName,
    executable: runtime.Executable.fromPartial({
        command: "/usr/bin/sleep 4",
        description: "Sleep for 4 seconds",
        name: "sleep-4"
    })
})
helpers.print(started)

// wait for sleep to complete
const sleep = async (waitTime: number) =>
    new Promise(resolve =>
        setTimeout(resolve, waitTime));

sleep(5000);

// [ Stop ]
let stopped = await cells.stop(<runtime.CellServiceStopRequest>{
    cellName,
    executableName: "sleep-4",
})
helpers.print(stopped)

// [ Free ]
let freed = await cells.free(<runtime.CellServiceFreeRequest>{
    cellName
});
helpers.print(freed)