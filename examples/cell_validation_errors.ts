// [ Free ]
import * as aurae from "../auraescript/gen/aurae.ts";
import * as cells from "../auraescript/gen/cells.ts";

let client = await aurae.createClient();
const cellService = new cells.CellServiceClient(client);

//// REGEX violation
await cellService.allocate(<cells.CellServiceAllocateRequest>{
    cell: cells.Cell.fromPartial({
        name: "ae-no_underscore"
    })
});
