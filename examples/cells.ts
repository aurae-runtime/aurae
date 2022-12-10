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
        name: "my-cell",
        cpus: "2"
    })
});
helpers.print(allocated)


// [ Start ]
let started = await cells.start(<runtime.StartCellRequest>{
    executable: runtime.Executable.fromPartial({
        cellName: "my-cell",
        command: "sleep 45",
        description: "Read the filesystem tab file.",
        name: "cat-fstab"
    })
})
helpers.print(started)


// [ Stop ]
let stopped = await cells.stop(<runtime.StopCellRequest>{
    cellName: "my-cell",
    executableName: "cat-fstab",
})
helpers.print(stopped)


// [ Free ]
// let freed = await cells.free(<runtime.FreeCellRequest>{
//     cellName: "my-cell"
// });
// helpers.print(freed)
































// // @ts-ignore
// import * as helpers from "../auraescript/gen/helpers.ts";
// // @ts-ignore
// import * as runtime from "../auraescript/gen/runtime.ts";
//
// let cells = new runtime.CellServiceClient();
//
// // Define Cell
// let cell = <runtime.AllocateCellRequest>{
//     cell: runtime.Cell.fromPartial({
//         name: "my-cell",
//         cpus: "2"
//     })};
//
// // Allocate cell
// cells.allocate(cell).then(r => {
//     helpers.print(r);
//     let cell_name = r.cell_name;
//
//     // Start executable within a cell
//     cells.start(<runtime.StartCellRequest>{
//         executable: runtime.Executable.fromPartial({
//             name: "fstab-reader",
//             command: "cat /etc/fstab",
//             description: "Used to read the current filesystem tab (fstab) from disk.",
//             cellName: "my-cell",
//         })
//     }).then(r =>  {
//         helpers.print(r)
//         // Stop executable within a cell
//         cells.stop(<runtime.StopCellRequest>{
//             cellName: "my-cell",
//             executableName: "fstab-reader",
//         }).then(r => helpers.print(r));
//     });
//
//
//     // Free cell
//     cells.free(<runtime.FreeCellRequest>{
//         cell_name: cell_name,
//     }).then(r => helpers.print(r));
// });
