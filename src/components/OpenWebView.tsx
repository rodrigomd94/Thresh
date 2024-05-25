import { invoke } from '@tauri-apps/api/tauri';
import { WebviewWindow } from '@tauri-apps/api/window'
import { useEffect, useState } from 'react';

import { C } from "../cml/init.js";

import { Web3AuthWalletAPI, createWalletFromMnemonic, getAddressesFromKeys, testMenmonic } from '../cip30/WalletApi.js';
import { listenToEvents } from '../cip30/TestWalletApi.js';
//import { script } from '../cip30/InjectScript.js';

const OpenWebview = () => {
    const [website, setWebsite] = useState('https://app.minswap.org');
    const [webview, setWebview] = useState<any>(null)
    const openBrowser = async () => {
        const webview = new WebviewWindow('webview', {
            url: website,
        })
        // since the webview window is created asynchronously,
        // Tauri emits the `tauri://created` and `tauri://error` to notify you of the creation response
        webview.once('tauri://created', function () {
           // listenToEvents(webview)
            injectJavaScript(webview)
            // webview window successfully created
            // webview.show()
        })
        webview.once('tauri://error', function () {
            // an error occurred during webview window creation
        })
    };

    const injectJavaScript = async (webview: WebviewWindow) => {
        console.log("Injecting JS")
        const privKey = await createWalletFromMnemonic(testMenmonic, "Mainnet")
        const walletAPI = new Web3AuthWalletAPI(privKey.paymentKey, privKey.stakeKey, "Mainnet", "https://cardano-preprod.blockfrost.io/api/v0", "mainnetRYnsFXoxIeqmPhpUcHmS91I0DfADfJon")
        await invoke('init_wallet_api', { window:'webview' })
    }

    return (
        <div>
            <button onClick={openBrowser}>
                Open Browser
            </button>
            <input value={website} onChange={(e) => setWebsite(e.target.value)} type="text" id="input" />
        </div>
    );
};

export default OpenWebview;
