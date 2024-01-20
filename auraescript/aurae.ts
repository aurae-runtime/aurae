type CreateClientDefault = {
    kind: "default";
};
type CreateClientPath = {
    kind: "path";
    path: string;
};
type CreateClientOpts = {
    kind: "opts";
    ca_crt: string;
    client_crt: string;
    client_key: string;
    socket: string;
};
type CreateClient =
    | CreateClientDefault
    | CreateClientPath
    | CreateClientOpts;

export function createClient(opts: CreateClient = { kind: "default" }): Promise<number> {
    if (typeof opts.kind === "undefined") {
        // resolve kind
        if ("path" in opts) {
            opts.kind = "path";
        } else if ("ca_crt" in opts) {
            opts.kind = "opts";
        }
    }
    let config;
    switch (opts.kind) {
        case "default": {
            config = Deno[Deno.internal].core.ops.as__aurae_config__try_default();
            break;
        }
        case "path": {
            config = Deno[Deno.internal].core.ops.as__aurae_config__parse_from_file(opts.path);
            break;
        }
        case "opts": {
            config = Deno[Deno.internal].core.ops.as__aurae_config__from_options(
                opts.ca_crt, opts.client_crt, opts.client_key, opts.socket
            );
            break;
        }
        default: {
            const _exhaustiveCheck: never = opts;
            return _exhaustiveCheck;
        }
    }
    // @ts-ignore
    return Deno[Deno.internal].core.ops.as__client_new(config);
}
