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
const nestedCellName = "ae-sleeper-cell/nested-sleeper"
const cellName = "ae-sleeper-cell";

// [ Allocate ]
let allocated = await cellService.allocate(<cells.CellServiceAllocateRequest>{
    cell: cells.Cell.fromPartial({
        name: cellName,
        cpu: cells.CpuController.fromPartial({
            weight: 2, // Percentage of CPUs
            max: 400 * (10 ** 3), // 0.4 seconds in microseconds
        }),
    })
});
aurae.print('Allocated:', allocated)

// [ Start ]
let started = await cellService.start(<cells.CellServiceStartRequest>{
    cellName,
    executable: cells.Executable.fromPartial({
        command: "/usr/bin/sleep 42",
        description: "Sleep for 42 seconds",
        name: "sleep-42"
    })
})
aurae.print('Started:', started)

// [ Allocate nested ]
let nested_allocated = await cellService.allocate(<cells.CellServiceAllocateRequest>{
    cell: cells.Cell.fromPartial({
        name: nestedCellName,
        cpu: cells.CpuController.fromPartial({
            weight: 2, // Percentage of CPUs
            max: 400 * (10 ** 3), // 0.4 seconds in microseconds
        }),
    })
});
aurae.print('Allocated Nested:', nested_allocated)

// [ List cellService ]
let listed = await cellService.list(<cells.CellServiceListRequest>{})
aurae.print('Listed:', listed)

// [ Stop ]
let stopped = await cellService.stop(<cells.CellServiceStopRequest>{
    cellName,
    executableName: "sleep-42",
})
aurae.print('Stopped:', stopped)

// [ Free ]
let freed = await cellService.free(<cells.CellServiceFreeRequest>{
    cellName
});
aurae.print('Freed:', freed)
