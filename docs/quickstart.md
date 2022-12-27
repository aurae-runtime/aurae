# Aurae Quickstart

Now that you have [built Aurae from source](/build) you can begin using Aurae.

### Running the Daemon 

Aurae will run on any system, even if `systemd` or another init daemon is currently active. 

```bash 
sudo -E auraed -v
```

### Running your first Cell

All executables in Aurae are ran in an Aurae cell which is just an isolation boundary for a regular executable to run in.

Take the following example code which will create a new cell called `sleeper-cell` which runs with a small CPU quota (time allowed for the process to execute in) and only has access to 2 of the available cores on your system.

```typescript
// create-cell.ts
import * as helpers from "../auraescript/gen/helpers.ts";
import * as runtime from "../auraescript/gen/runtime.ts";
let cells = new runtime.CellServiceClient();
let allocated = await cells.allocate(<runtime.AllocateCellRequest>{
    cell: runtime.Cell.fromPartial({
        cpuQuota: 400 * (10 ** 3), // 0.4 seconds in microseconds
        cpuShares: 2, // Percentage of CPUs
        name: "sleeper-cell",
    })
});
helpers.print(allocated)
```

The script can be executed locally against a running `auraed` daemon as long as you have [certificates](/certs) installed and configured properly. 

Once a cell is allocated it will continue to reserve the required resources and persist until the system is rebooted or until another action destroys the cell. 

Once a cell is created, any amount of nested executables can be executed directly inside the cell. All executables inside a given cell have access to other executables network, storage, and process communication.

```typescript
// run-sleep-in-cell.ts
import * as helpers from "../auraescript/gen/helpers.ts";
import * as runtime from "../auraescript/gen/runtime.ts";
let cells = new runtime.CellServiceClient();
let started = await cells.start(<runtime.StartExecutableRequest>{
    cellName: "sleeper-cell", // Same name must map back to cell!
    executable: runtime.Executable.fromPartial({
        command: "/usr/bin/sleep",
        args: ["42"],
        description: "Sleep for 42 seconds",
        name: "sleep-42"
    })
})
helpers.print(started)
```

Note that in this example the command tries to sleep for longer than the quota allows, and thus is terminated by the kernel.