// Check connection status with background script
async function checkConnection() {
  try {
    const response = await chrome.runtime.sendMessage({ type: 'check_connection' });
    updateStatus(response.connected, response.message);
  } catch (error) {
    updateStatus(false, 'Extension error: ' + error.message);
  }
}

function updateStatus(connected, message) {
  const statusEl = document.getElementById('status');
  const statusText = document.getElementById('status-text');
  const connectBtn = document.getElementById('connect-btn');
  const nativeInfo = document.getElementById('native-app-info');
  
  statusText.textContent = message || (connected ? 'Connected to Thresh app' : 'Thresh app not found');
  
  if (connected) {
    statusEl.classList.remove('disconnected');
    statusEl.classList.add('connected');
    connectBtn.textContent = 'Reconnect';
    connectBtn.disabled = false;
    nativeInfo.textContent = 'Native messaging host: com.cardano.thresh';
  } else {
    statusEl.classList.remove('connected');
    statusEl.classList.add('disconnected');
    connectBtn.textContent = 'Connect to Thresh App';
    connectBtn.disabled = false;
    nativeInfo.textContent = 'Make sure the Thresh wallet app is installed and registered.';
  }
}

// Connect button handler
document.getElementById('connect-btn').addEventListener('click', async () => {
  const btn = document.getElementById('connect-btn');
  btn.disabled = true;
  btn.textContent = 'Connecting...';
  
  try {
    const response = await chrome.runtime.sendMessage({ type: 'connect' });
    updateStatus(response.connected, response.message);
  } catch (error) {
    updateStatus(false, 'Failed to connect: ' + error.message);
  }
});

// Initial check
checkConnection();

// Check periodically
setInterval(checkConnection, 5000);