// [ Free ]
import * as runtime from "../auraescript/gen/runtime.ts";
import * as helpers from "../auraescript/gen/helpers.ts";

let cells = new runtime.CellServiceClient();

//// REQUIRED
// let freed = await cells.free(<runtime.FreeCellRequest>{
//     cellName: ""
// });

//// REGEX violation
let freed = await cells.free(<runtime.FreeCellRequest>{
    cellName: "nope_nope"
});
helpers.print(freed)