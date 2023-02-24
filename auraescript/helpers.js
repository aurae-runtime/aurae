function print(...values) {
    // @ts-ignore
    Deno.core.print(values.map(toString).join(' ') + "\n");
}

function toString(value) {
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

function auraescript_client(
    ca_crt, client_crt, client_key, socket,
) {
    // @ts-ignore
    return Deno.core.ops.auraescript_client(ca_crt, client_crt, client_key, socket);
}
