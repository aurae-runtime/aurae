Aurae offers a Turing complete scripting language built on top of TypeScript called [AuraeScript](https://github.com/aurae-runtime/aurae/tree/main/auraescript). AuraeScript embeds the [Deno](https://deno.land) source code directly, and offers a remote client and SDK to interface directly with Aurae remotely. The AuraeScript library is automatically generated from the `.proto` files defined in the [Aurae Standard Library](https://aurae.io/stdlib/).

Valid TypeScript files can be leveraged to replace static manifests, as well as interact directly with a running system.

```typescript
#!/usr/bin/env auraescript
let cells = new runtime.CellServiceClient();

let allocated = await cells.allocate(<runtime.AllocateCellRequest>{
    cell: runtime.Cell.fromPartial({
        name: "my-cell",
        cpus: "2"
    })
});

let started = await cells.start(<runtime.StartExecutableRequest>{
    executable: runtime.Executable.fromPartial({
        cellName: "my-cell",
        command: "sleep 4000",
        description: "Sleep for 4000 seconds",
        name: "sleep-4000"
    })
})
```

See the [Full list of working examples](https://github.com/aurae-runtime/aurae/tree/main/examples) for more!