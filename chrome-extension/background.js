// Background script - Handles native messaging with Tauri app

const NATIVE_HOST_NAME = 'com.cardano.tauri_wallet';
let nativePort = null;
let isConnected = false;
let pendingRequests = new Map(); // Store pending requests
let enabledOrigins = new Set(); // Track enabled origins

// Connect to native messaging host
function connectNative() {
  try {
    console.log('Attempting to connect to native host:', NATIVE_HOST_NAME);
    nativePort = chrome.runtime.connectNative(NATIVE_HOST_NAME);
    
    if (!nativePort) {
      console.error('Failed to create native port');
      isConnected = false;
      return;
    }
    
    nativePort.onMessage.addListener((response) => {
      console.log('Received from native app:', response);
      
      // Handle response for pending request
      if (response.requestId && pendingRequests.has(response.requestId)) {
        const { resolve } = pendingRequests.get(response.requestId);
        pendingRequests.delete(response.requestId);
        resolve(response);
      }
    });
    
    nativePort.onDisconnect.addListener(() => {
      console.log('Native port disconnected');
      const error = chrome.runtime.lastError;
      if (error) {
        console.error('Native messaging error:', error);
        console.error('Error message:', error.message);
      }
      isConnected = false;
      nativePort = null;
      
      // Reject all pending requests
      for (const [_, { reject }] of pendingRequests) {
        reject(new Error(error ? error.message : 'Native messaging host disconnected'));
      }
      pendingRequests.clear();
    });
    
    // Send initial ping to verify connection
    sendNativeMessage({ type: 'ping' }).then(response => {
      if (response.type === 'pong') {
        isConnected = true;
        console.log('Successfully connected to native host');
      }
    }).catch(err => {
      console.error('Failed to ping native host:', err);
      isConnected = false;
    });
    
  } catch (error) {
    console.error('Failed to connect to native host:', error);
    isConnected = false;
  }
}

// Send message to native app
function sendNativeMessage(message) {
  return new Promise((resolve, reject) => {
    if (!nativePort) {
      reject(new Error('Not connected to native host'));
      return;
    }
    
    const requestId = Date.now() + Math.random();
    const messageWithId = { ...message, requestId };
    
    pendingRequests.set(requestId, { resolve, reject });
    
    // Set timeout for request
    setTimeout(() => {
      if (pendingRequests.has(requestId)) {
        pendingRequests.delete(requestId);
        reject(new Error('Request timeout'));
      }
    }, 30000); // 30 second timeout
    
    try {
      nativePort.postMessage(messageWithId);
    } catch (error) {
      pendingRequests.delete(requestId);
      reject(error);
    }
  });
}

// Handle messages from content script and popup
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
  console.log('Received message:', request);
  
  // Handle different message types
  switch (request.type) {
    case 'check_connection':
      sendResponse({ 
        connected: isConnected, 
        message: isConnected ? 'Connected to Tauri app' : 'Not connected' 
      });
      break;
      
    case 'connect':
      if (!nativePort) {
        connectNative();
      }
      // Check connection status after a short delay
      setTimeout(() => {
        sendResponse({ 
          connected: isConnected, 
          message: isConnected ? 'Connected to Tauri app' : 'Failed to connect' 
        });
      }, 1000);
      return true; // Keep channel open for async response
      
    case 'cip30_request':
      handleCIP30Request(request.data, sender.origin || sender.url)
        .then(response => sendResponse(response))
        .catch(error => sendResponse({ error: error.message }));
      return true; // Keep channel open for async response
      
    default:
      sendResponse({ error: 'Unknown request type' });
  }
});

// Handle CIP-30 API requests
async function handleCIP30Request(request, origin) {
  // Special handling for enable/isEnabled
  if (request.type === 'isEnabled') {
    return enabledOrigins.has(request.origin || origin);
  }
  
  if (request.type === 'enable') {
    // For now, auto-approve all enable requests
    // In production, you'd show a user consent dialog
    enabledOrigins.add(request.origin || origin);
    return { enabled: true };
  }
  
  // Check if origin is enabled
  if (!enabledOrigins.has(origin) && request.type !== 'enable') {
    throw new Error('Wallet not enabled for this origin');
  }
  
  // Forward request to native app
  if (!isConnected) {
    connectNative();
    // Wait a bit for connection
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    if (!isConnected) {
      throw new Error('Cannot connect to Tauri wallet app');
    }
  }
  
  try {
    const response = await sendNativeMessage({
      type: 'cip30',
      method: request.type,
      params: request
    });
    
    if (response.error) {
      throw new Error(response.error);
    }
    
    // Check if data exists (including 0, false, null values)
    return response.hasOwnProperty('data') ? response.data : response;
  } catch (error) {
    console.error('Native messaging error:', error);
    throw error;
  }
}

// Initialize connection on startup
connectNative();