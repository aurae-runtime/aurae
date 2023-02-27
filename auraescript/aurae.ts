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

export function createClient(
    path_or_ca_crt?: string, client_crt?: string, client_key?: string, socket?: string,
): AuraeClient {
    let config;
    // Number of params
    // 0 -> default config
    if (typeof path_or_ca_crt === 'undefined') {
        config = Deno.core.ops.as__aurae__config__try_default();
    // 1 -> file path
    } else if (typeof client_crt === 'undefined') {
        config = Deno.core.ops.as__aurae__config__parse_from_file(path_or_ca_crt);
    // 4 -> options
    } else {
        config = Deno.core.ops.as__aurae__config__from_options(path_or_ca_crt, client_crt, client_key, socket);
    }
    // @ts-ignore
    return Deno.core.ops.as__client_new(config);
}
