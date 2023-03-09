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
import * as cells from "../auraescript/gen/cells.ts";
import * as aurae from "../auraescript/gen/aurae.ts";


let client = await aurae.createClient();

let cellService = new cells.CellServiceClient(client);
let cellName = "ae-echo-cell";

// [ Allocate ]
let allocated = await cellService.allocate(<cells.CellServiceAllocateRequest>{
    cell: cells.Cell.fromPartial({
        cpu: cells.CpuController.fromPartial({
            weight: 2, // Percentage of CPUs
            max: 400 * (10 ** 3), // 0.4 seconds in microseconds
        }),
        name: cellName,
    })
});
//aurae.print(allocated)

// [ Start ]
let started_out = await cellService.start(<cells.CellServiceStartRequest>{
    cellName,
    executable: cells.Executable.fromPartial({
        command: "/usr/bin/echo 'hello world'",
        description: "outputs a message to stdout",
        name: "echo-stdout"
    })
})
//aurae.print(started_out)

// [ Start ]
let started_err = await cellService.start(<cells.CellServiceStartRequest>{
    cellName,
    executable: cells.Executable.fromPartial({
        command: "/usr/bin/echo 'hello world' 1>&2",
        description: "outputs a message to stderr",
        name: "echo-stderr"
    })
})

// [ Stop ]
let stopped_out = await cellService.stop(<cells.CellServiceStopRequest>{
    cellName,
    executableName: "echo-stdout",
})
// [ Stop ]
let stopped_err = await cellService.stop(<cells.CellServiceStopRequest>{
    cellName,
    executableName: "echo-stderr",
})
//aurae.print(stopped)

// [ Free ]
let freed = await cellService.free(<cells.CellServiceFreeRequest>{
    cellName
});
//aurae.print(freed)
