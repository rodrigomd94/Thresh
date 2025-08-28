# Tauri Cardano Wallet Extension

A Chrome browser extension that implements CIP-30 (Cardano dApp connector standard) by communicating with a locally running Tauri application via Native Messaging.

## Architecture

- **Chrome Extension**: Injects the CIP-30 wallet API into web pages and handles communication with the Tauri app
- **Tauri App**: Runs locally and handles all wallet operations (currently with mock implementations)
- **Communication**: Uses Chrome's Native Messaging API for secure communication between extension and local app

## Project Structure

```
tauri-wallet-extension/
├── chrome-extension/        # Chrome extension source
│   ├── manifest.json       # Extension manifest (V3)
│   ├── content.js          # Injects CIP-30 API into pages
│   ├── background.js       # Handles native messaging
│   ├── popup.html/js       # Extension popup UI
│   └── icons/              # Extension icons
├── cip-30-wallet/          # Tauri application
│   └── src-tauri/
│       └── src/
│           ├── main.rs              # Entry point
│           ├── lib.rs               # Main app logic
│           ├── native_messaging.rs  # Native messaging handler
│           └── cip30/               # CIP-30 implementation
└── install-native-host.sh  # Installation script
```

## Installation

### Prerequisites

- Chrome or Chromium-based browser
- Rust and Cargo installed
- Node.js and npm (for Tauri frontend)

### Step 1: Build and Install the Tauri App

```bash
# Build the Tauri app
cd cip-30-wallet
npm install
npm run build
```

### Step 2: Load the Chrome Extension

1. Open Chrome and navigate to `chrome://extensions/`
2. Enable "Developer mode" (toggle in top right)
3. Click "Load unpacked" and select the `chrome-extension` directory
4. Note the extension ID shown on the extension card

### Step 3: Install Native Messaging Host

Run the installation script with your extension ID:

```bash
./install-native-host.sh YOUR_EXTENSION_ID_HERE
```

This script will:
- Build the Tauri app in release mode
- Install the binary to `/usr/local/bin/`
- Create the native messaging manifest in the correct location
- Configure the extension ID for secure communication

## Usage

### For Users

1. Click the extension icon in Chrome toolbar
2. Click "Connect to Tauri App" to establish connection
3. Visit any Cardano dApp that supports CIP-30
4. The dApp should detect "Tauri Wallet" as an available wallet

### For dApp Developers

The wallet is available at `window.cardano.thresh` and implements the full CIP-30 API:

```javascript
// Check if wallet is available
if (window.cardano && window.cardano.thresh) {
  const wallet = window.cardano.thresh;
  
  // Enable wallet (request access)
  const api = await wallet.enable();
  
  // Use wallet API
  const networkId = await api.getNetworkId();
  const balance = await api.getBalance();
  const utxos = await api.getUtxos();
  // ... etc
}
```

## CIP-30 API Methods

Currently implemented (with mock data):

- `enable()` - Request wallet access
- `isEnabled()` - Check if wallet is enabled for the site
- `getExtensions()` - Get supported CIP extensions
- `getNetworkId()` - Get network ID (0=testnet, 1=mainnet)
- `getUtxos()` - Get unspent transaction outputs
- `getBalance()` - Get total wallet balance
- `getUsedAddresses()` - Get used wallet addresses
- `getUnusedAddresses()` - Get unused wallet addresses  
- `getChangeAddress()` - Get change address
- `getRewardAddresses()` - Get stake reward addresses
- `signTx()` - Sign a transaction
- `signData()` - Sign arbitrary data
- `submitTx()` - Submit a transaction to the network

## Development

### Chrome Extension Development

1. Make changes to files in `chrome-extension/`
2. Click the refresh icon on the extension card in `chrome://extensions/`

### Tauri App Development

```bash
cd cip-30-wallet
npm run tauri dev
```

For native messaging testing:
```bash
cd cip-30-wallet/src-tauri
cargo build
./target/debug/cip-30-wallet --native-messaging
```

### Testing Native Messaging

Enable debug logging by viewing background script console:
1. Go to `chrome://extensions/`
2. Click "background page" link on extension card
3. Check console for native messaging logs

## Troubleshooting

### Extension can't connect to Tauri app

1. Check that the native messaging host is installed correctly:
   ```bash
   ls ~/.config/google-chrome/NativeMessagingHosts/com.cardano.thresh.json
   ```

2. Verify the extension ID in the manifest matches your extension

3. Check Chrome's native messaging logs:
   - Start Chrome from terminal with: `google-chrome --enable-logging=stderr --v=1`
   - Look for native messaging errors

### "Not a terminal" error during installation

The Tauri app is built manually in the installation script, bypassing the interactive `create-tauri-app` tool.

## Future Enhancements

- Replace mock implementations with real Cardano wallet functionality using:
  - [Pallas](https://github.com/txpipe/pallas) - Rust library for Cardano
  - [utxoRPC](https://github.com/utxorpc/rust-sdk) - For blockchain queries
- Add secure key management
- Implement transaction building and signing
- Add support for Cardano native tokens
- Implement hardware wallet support