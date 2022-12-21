// [ Free ]
import * as runtime from "../auraescript/gen/runtime.ts";

const cells = new runtime.CellServiceClient();

//// REQUIRED
// await cells.allocate(<runtime.AllocateCellRequest>{
//     cell: runtime.Cell.fromPartial({
//         name: ""
//     })
// });

//// REGEX violation
await cells.allocate(<runtime.AllocateCellRequest>{
    cell: runtime.Cell.fromPartial({
        name: "nope_nope"
    })
});
