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
