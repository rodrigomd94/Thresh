
import { listen, emit } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/tauri';
import { WebviewWindow } from '@tauri-apps/api/window';

(window as any).cardano = {
    nami: {
        enable: async function () {
            return {
                getBalance: async() => await invokeScript('get_balance'),
                getChangeAddress: () => invokeScript('get_change_address'),
                getCollateral: () => invokeScript('get_collateral'),
                getExtensions: () => invokeScript('get_extensions'),
                getNetworkId: () => invokeScript('get_network_id'),
               // getPubDRepKey: () => invokeScript('get_pubDRepKey'),
               // getRegisteredPubStakeKeys: () => invokeScript('getRegisteredPubStakeKeys'),
                getRewardAddresses: () => invokeScript('get_reward_addresses'),
                //getUnregisteredPubStakeKeys: () => invokeScript('get_unregisteredPubStakeKeys'),
                getUnusedAddresses: () => invokeScript('get_unused_addresses'),
                getUsedAddresses: () => invokeScript('get_used_addresses'),
                getUtxos: () => invokeScript('get_utxos'),
                signData: (addr: any, payload: any) => invokeScript('sign_data', addr, payload),
                signTx: (tx: any) => invokeScript('sign_tx', tx),
                submitTx: (tx: any) => invokeScript('submit_tx', tx)
            };
        }
    }
};

const invokeScript = async (script: string, ...params: any) => {
    console.log({params})
    return await invoke(script, params);
}
