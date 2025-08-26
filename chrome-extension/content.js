// Content script - Injects CIP-30 API into web pages

// Create a custom event for communication between injected script and content script
const WALLET_NAME = 'tauri_wallet';

// Inject the wallet API script into the page
function injectScript() {
  const script = document.createElement('script');
  script.src = chrome.runtime.getURL('injected-script.js');
  script.onload = function() {
    this.remove();
  };
  
  (document.head || document.documentElement).appendChild(script);
}

// Listen for messages from the injected script
window.addEventListener('tauri_wallet_request', async (event) => {
  const request = event.detail;
  
  try {
    // Forward request to background script
    const response = await chrome.runtime.sendMessage({
      type: 'cip30_request',
      data: request
    });
    
    // Send response back to injected script
    window.dispatchEvent(new CustomEvent('tauri_wallet_response', {
      detail: {
        messageId: request.messageId,
        data: response
      }
    }));
  } catch (error) {
    // Send error back to injected script
    window.dispatchEvent(new CustomEvent('tauri_wallet_response', {
      detail: {
        messageId: request.messageId,
        data: { error: error.message }
      }
    }));
  }
});

// Debug logging
console.log('[Tauri Wallet] Content script loaded on:', window.location.href);

// Inject script as soon as possible
injectScript();

// Debug: Check if injection worked
setTimeout(() => {
  console.log('[Tauri Wallet] window.cardano after injection:', window.cardano);
  if (window.cardano && window.cardano.tauri_wallet) {
    console.log('[Tauri Wallet] tauri_wallet found:', window.cardano.tauri_wallet);
  } else {
    console.log('[Tauri Wallet] tauri_wallet NOT found');
  }
}, 100);