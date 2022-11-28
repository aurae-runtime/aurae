export function print(value) {
    // @ts-ignore
    Deno.core.print(toString(value));
}

function toString(value: any): string {
    if (
        typeof value === 'object' &&
        !Array.isArray(value) &&
        value !== null
    ) {
        return JSON.stringify(value, null, 2) + "\n";
    } else {
        return value?.toString() + "\n";
    }
}