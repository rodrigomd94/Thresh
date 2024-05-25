/// <reference path="./cardano_message_signing.generated.d.ts" />

const C = await import('@libs/libs/cardano_multiplatform_lib/cardano_multiplatform_lib.generated.js')
const M = await import('@libs/cardano_message_signing/cardano_message_signing.generated.js')

async function unsafeInstantiate(module: any) {
    try {
        await module.instantiate();
    } catch (_e) {
        // This only ever happens during SSR rendering
    }
}
await Promise.all([
    unsafeInstantiate(C),
    unsafeInstantiate(M),
]);


export { C, M }