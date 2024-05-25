import * as wasm from '@libs/libs/cardano_multiplatform_lib/cardano_multiplatform_lib.generated.js';
import * as wasm2 from '@libs/cardano_message_signing/cardano_message_signing.generated.js';

/**
 * Loads the WASM modules
 */

class Loader {
    _wasm:any = null;
    _wasm2:any = null;
    async load() {
        if (this._wasm && this._wasm2) return;
        try {
            await wasm.instantiate();
            await wasm2.instantiate();
        } catch (_e) {
            // Only happens when running with Jest (Node.js)
        }
        /**
         * @private
         */
        this._wasm = wasm;
        /**
         * @private
         */
        this._wasm2 = wasm2;
    }

    get Cardano() {
        return this._wasm;
    }

    get Message() {
        return this._wasm2;
    }
}

export default new Loader();