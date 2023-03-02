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
let cellName = "ae-sleeper-cell";

// [ Allocate ]
let allocated = await cellService.allocate(<cells.CellServiceAllocateRequest>{
    cell: cells.Cell.fromPartial({
        memory: cells.MemoryController.fromPartial({
            high: 50000, // 50k
            max: 100000, // 100k
        }),
        name: cellName,
    })
});
aurae.print(allocated)

// [ Start ]
let started = await cellService.start(<cells.CellServiceStartRequest>{
    cellName,
    executable: cells.Executable.fromPartial({
        command: "/usr/bin/sleep 2",
        description: "Sleep for 2 seconds",
        name: "sleep-2"
    })
})
aurae.print(started)

// wait for sleep to complete
const sleep = async (waitTime: number) =>
    new Promise(resolve =>
        setTimeout(resolve, waitTime));

sleep(3000);

// [ Stop ]
let stopped = await cellService.stop(<cells.CellServiceStopRequest>{
    cellName,
    executableName: "sleep-2",
})
aurae.print(stopped)

// [ Free ]
let freed = await cellService.free(<cells.CellServiceFreeRequest>{
    cellName
});
aurae.print(freed)
