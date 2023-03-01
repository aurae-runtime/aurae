export function print(...values: any[]) {
    // @ts-ignore
    Deno.core.print(values.map(toString).join(' ') + "\n");
}

function toString(value: any): string {
    if (
        typeof value === 'object' &&
        !Array.isArray(value) &&
        value !== null
    ) {
        return JSON.stringify(value, null, 2);
    } else {
        return value?.toString();
    }
}

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
            config = Deno.core.ops.as__aurae_config__try_default();
            break;
        }
        case "path": {
            config = Deno.core.ops.as__aurae_config__parse_from_file(opts.path);
            break;
        }
        case "opts": {
            config = Deno.core.ops.as__aurae_config__from_options(
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
    return Deno.core.ops.as__client_new(config);
}
