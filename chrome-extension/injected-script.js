(function() {
  // Initialize window.cardano if it doesn't exist
  if (!window.cardano) {
    window.cardano = {};
  }

  // Create wallet API object
  const tauriWallet = {
    apiVersion: '1.0.0',
    name: 'Tauri Wallet',
    icon: 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAACAAAAAgCAYAAABzenr0AAAABHNCSVQICAgIfAhkiAAAAAlwSFlzAAAA7AAAAOwBeShxvQAAABl0RVh0U29mdHdhcmUAd3d3Lmlua3NjYXBlLm9yZ5vuPBoAAAGRSURBVFiF7Zi/SsRAFMQ/V3gIgoUgCIJgYSEIgoWNhY2FjYWNhZ2dYGNhY2FjYWNhY2Hj38bGwsbCxsLGQhAEQRAEQRAEQfC4goV7l71kk+zuzSaZB8MdZHZnZt/bTQKEYRhGsZTdDqAITANTwCQwDowBo8Bwa0yCQeBLa3xpnfuo9b8BjcAqAFeS5pKMOkkNSTVJD5Juku4k3Um6k/Qg6SGI9yBpIalSYI5xkuaSSgXPl8wAcBJkfwFo9pm9/+xJTWrWBQyA48D5DbgGNoEtYBvYaY2dwLkBnLVyZaoKmATGW88jwAjQC1SSJm2R1PNhJEnhEHJRdbuJdgOo0rwfVGk20VUguMmcB849tJvoCrAVODfIDQOdBvbx7kBbgQfavyi9NOQasEfzajQCrHa64hE6DezQbKBT4DJLoLcFukUD7QJPeSfwtkBNUr1NQWuSupJ2MJJxRruBX0nveSfJ00CDuMQlLnGJS1ziEpe4xOVfE2fm/5FYAB6BF+AtS9Ck7H4QDMOIL1/iipQoefgdLgAAAABJRU5ErkJggg==',
    supportedExtensions: [],
    
    // Check if wallet is already enabled
    isEnabled: async function() {
      return await sendToContentScript({
        type: 'isEnabled',
        origin: window.location.origin
      });
    },

    // Enable wallet (request access)
    enable: async function(extensions = {}) {
      const result = await sendToContentScript({
        type: 'enable',
        origin: window.location.origin,
        extensions: extensions.extensions || []
      });
      
      if (result.error) {
        throw new Error(result.error);
      }
      
      // Return the full API
      return createFullAPI();
    }
  };

  // Full API (returned after enable)
  function createFullAPI() {
    return {
      // Get enabled extensions
      getExtensions: async () => {
        return await sendToContentScript({ type: 'getExtensions' });
      },

      // Network ID (0 = testnet, 1 = mainnet)
      getNetworkId: async () => {
        return await sendToContentScript({ type: 'getNetworkId' });
      },

      // Get UTXOs
      getUtxos: async (amount = undefined, paginate = undefined) => {
        return await sendToContentScript({ 
          type: 'getUtxos',
          amount: amount,
          paginate: paginate
        });
      },

      // Get wallet balance
      getBalance: async () => {
        return await sendToContentScript({ type: 'getBalance' });
      },

      // Get used addresses
      getUsedAddresses: async (paginate = undefined) => {
        return await sendToContentScript({ 
          type: 'getUsedAddresses',
          paginate: paginate
        });
      },

      // Get unused addresses
      getUnusedAddresses: async () => {
        return await sendToContentScript({ type: 'getUnusedAddresses' });
      },

      // Get change address
      getChangeAddress: async () => {
        return await sendToContentScript({ type: 'getChangeAddress' });
      },

      // Get reward addresses
      getRewardAddresses: async () => {
        return await sendToContentScript({ type: 'getRewardAddresses' });
      },

      // Sign transaction
      signTx: async (tx, partialSign = false) => {
        const result = await sendToContentScript({ 
          type: 'signTx',
          tx: tx,
          partialSign: partialSign
        });
        
        if (result.error) {
          throw new Error(result.error);
        }
        
        return result.witness;
      },

      // Sign data
      signData: async (addr, payload) => {
        const result = await sendToContentScript({ 
          type: 'signData',
          addr: addr,
          payload: payload
        });
        
        if (result.error) {
          throw new Error(result.error);
        }
        
        return result.signature;
      },

      // Submit transaction
      submitTx: async (tx) => {
        const result = await sendToContentScript({ 
          type: 'submitTx',
          tx: tx
        });
        
        if (result.error) {
          throw new Error(result.error);
        }
        
        return result.txHash;
      }
    };
  }

  // Helper to send messages to content script
  async function sendToContentScript(data) {
    return new Promise((resolve) => {
      const messageId = Date.now() + Math.random();
      
      // Listen for response
      const handler = (event) => {
        if (event.detail && event.detail.messageId === messageId) {
          window.removeEventListener('tauri_wallet_response', handler);
          resolve(event.detail.data);
        }
      };
      
      window.addEventListener('tauri_wallet_response', handler);
      
      // Send message
      window.dispatchEvent(new CustomEvent('tauri_wallet_request', {
        detail: { ...data, messageId }
      }));
    });
  }

  // Register the wallet
  window.cardano.tauri_wallet = tauriWallet;

  // Dispatch event to notify that wallet is available
  window.dispatchEvent(new Event('cardano_wallet_loaded'));
  
  console.log('[Tauri Wallet] Injected wallet API successfully');
})();