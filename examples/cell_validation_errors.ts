// [ Free ]
import * as runtime from "../auraescript/gen/cells.ts";

const cells = new runtime.CellServiceClient();

//// REGEX violation
await cells.allocate(<runtime.CellServiceAllocateRequest>{
    cell: runtime.Cell.fromPartial({
        name: "ae-no_underscore"
    })
});
