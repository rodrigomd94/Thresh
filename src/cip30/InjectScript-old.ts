
import { listen, emit } from '@tauri-apps/api/event';
import { WebviewWindow } from '@tauri-apps/api/window';

(window as any).cardano = {
  nami: {
    enable: async function () {
      return {
        /* "getUnusedAddresses": ()=>["01b651e3ec8b29b74bff323cd32dbfea9c38d4434d30d9098e11de95e8d5f43d49f8a01021367b1394cf93918a8ae9d0c8f0176bef3e9d3e0a"], 
        "getChangeAddress":()=> "01b651e3ec8b29b74bff323cd32dbfea9c38d4434d30d9098e11de95e8d5f43d49f8a01021367b1394cf93918a8ae9d0c8f0176bef3e9d3e0a", 
        "getUsedAddresses": ()=> ["01b651e3ec8b29b74bff323cd32dbfea9c38d4434d30d9098e11de95e8d5f43d49f8a01021367b1394cf93918a8ae9d0c8f0176bef3e9d3e0a"], 
        "getRewardAddresses": ()=> ["e1d5f43d49f8a01021367b1394cf93918a8ae9d0c8f0176bef3e9d3e0a"], 
        "getNetworkId": ()=>1, */
        getBalance: () => sendMessage('getBalance'),
        getChangeAddress: () => sendMessage('getChangeAddress'),
        getCollateral: () => sendMessage('getCollateral'),
        getExtensions: () => sendMessage('getExtensions'),
        getNetworkId: () => sendMessage('getNetworkId'),
        getPubDRepKey: () => sendMessage('getPubDRepKey'),
        getRegisteredPubStakeKeys: () => sendMessage('getRegisteredPubStakeKeys'),
        getRewardAddresses: () => sendMessage('getRewardAddresses'),
        getUnregisteredPubStakeKeys: () => sendMessage('getUnregisteredPubStakeKeys'),
        getUnusedAddresses: () => sendMessage('getUnusedAddresses'),
        getUsedAddresses: () => sendMessage('getUsedAddresses'),
        getUtxos: () => sendMessage('getUtxos'),
        signData: (addr: any, payload: any) => sendMessage('signData', addr, payload),
        signTx: (tx: any) => sendMessage('signTx', tx),
        submitTx: (tx: any) => sendMessage('submitTx', tx)
      };
    }
  }
};

const subscribeToEvents = () => {
  for (const event of Object.keys((window as any).cardano.nami)) {
    listen(event, (payload) => {
      emit(`${event}-res`, payload);
    });
  }

}
function sendMessage(action: any, ...params: any) {

  const w = WebviewWindow.getByLabel("webview")
  //emit(action, params);
  return new Promise((resolve, reject) => {
    listen(`${action}-res`, (event) => {
      console.log("getBalance heereee", event)
    });
    console.log("sending message", { action })
    w!.emit(action, { message: 'hello' });
  });
}
