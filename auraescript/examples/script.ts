// @ts-ignore
import {AllocateCellRequest, Cell, CellServiceClient} from "../lib/runtime.ts";

// @ts-ignore
Deno.core.initializeAsyncOps();

let cells = new CellServiceClient();

cells.Allocate(<AllocateCellRequest>{
    cell: Cell.fromPartial({
        name: "test",
        cpus: "2"
    })
}).then(r => {
    // @ts-ignore
    Deno.core.print("done")
});