(function() {
  // Initialize window.cardano if it doesn't exist
  if (!window.cardano) {
    window.cardano = {};
  }

  // Create wallet API object
  const tauriWallet = {
    apiVersion: '1.0.0',
    name: 'Thresh',
    icon: 'data:image/png;base64,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',
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
          window.removeEventListener('thresh_response', handler);
          resolve(event.detail.data);
        }
      };
      
      window.addEventListener('thresh_response', handler);
      
      // Send message
      window.dispatchEvent(new CustomEvent('thresh_request', {
        detail: { ...data, messageId }
      }));
    });
  }

  // Register the wallet
  window.cardano.thresh = tauriWallet;

  // Dispatch event to notify that wallet is available
  window.dispatchEvent(new Event('cardano_wallet_loaded'));
  
  console.log('[Thresh] Injected wallet API successfully');
})();