# Thresh - A Desktop Cardano Wallet with Browser Extension

Thresh is a Cardano desktop wallet that exposes the CIP-30 API through a browser extension, enabling seamless interaction with Cardano dApps while maintaining the security and control of a locally running wallet.

## Overview

Thresh combines a Tauri-based desktop application with a Chrome browser extension to provide:

- **Desktop Wallet**: A secure, locally-running Cardano wallet built with Rust and Tauri
- **Browser Extension**: Implements the CIP-30 standard for dApp connectivity
- **Native Messaging**: Secure communication between browser and desktop app
- **Local Node Integration**: Uses Dolos, a lightweight Cardano data node, for blockchain interaction

## Key Features & Motivations

### 1. **Local Data Node Integration**
Thresh uses [Dolos](https://github.com/txpipe/dolos), a lightweight Cardano node that enables:
- Quick blockchain syncing with minimal resource usage
- Independence from external Cardano APIs
- Running your own devnet for local development
- Complete control over your blockchain data

### 2. **Rust Ecosystem Integration**
Built on Rust, Thresh can integrate powerful Cardano tooling:
- [Scrolls](https://github.com/txpipe/scrolls) for custom indexing solutions
- FROST libraries for multi-party computation (future)
- Direct integration with Pallas for Cardano primitives

### 3. **Enhanced Security & Privacy**
- All wallet operations happen locally on your machine
- No reliance on third-party services for transaction data
- Private keys never leave your device
- Desktop-level security features

### 4. **Flexible Architecture**
- Use as a standalone desktop wallet
- Enable browser extension for dApp interaction
- Run in isolated mode without browser connectivity
- Support for multiple networks including custom devnets

## Security

### Key Storage & Encryption

Thresh implements robust security measures for protecting your wallet:

- **Encryption**: ChaCha20Poly1305 authenticated encryption
- **Key Derivation**: Argon2 with 2500 iterations for password-based key derivation
- **Salt & Nonce**: Cryptographically secure random generation
- **Storage**: Encrypted wallet data stored locally with version control
- **Memory Safety**: Built in Rust to prevent common security vulnerabilities

Private keys and mnemonics are never stored in plain text and are encrypted using your wallet password.

## Installation & Setup

### Prerequisites

- Chrome or Chromium-based browser
- Rust and Cargo installed
- Node.js and npm
- Git

### Step 1: Clone the Repository

```bash
# Clone the repository
git clone https://github.com/your-repo/tauri-wallet-extension.git
cd tauri-wallet-extension

# Install dependencies
cd cip-30-wallet
npm install
cd ..
```

### Step 2: Load Chrome Extension

1. Open Chrome and navigate to `chrome://extensions/`
2. Enable "Developer mode" (toggle in top right)
3. Click "Load unpacked" and select the `chrome-extension` directory
4. Note the extension ID shown on the extension card

### Step 3: Build and Install Native Messaging Host

```bash
# Build the Tauri app and install native messaging host
./dev.sh rebuild YOUR_EXTENSION_ID_HERE
```

### Step 4: Install and Run Dolos

Install tx3up (Tx3 ecosystem manager):
```bash
# Install tx3up
curl -sSL https://tx3.dev/install.sh | sh

# Add to PATH (add to your shell config)
export PATH="$HOME/.tx3/bin:$PATH"
```

Run Dolos:
```bash
# Navigate to the dolos directory
cd ../dolos

# Start Dolos daemon
dolos daemon

# Dolos will sync with the configured network (mainnet/testnet/devnet)
```

### Step 5: Run the Desktop Wallet

The wallet runs as a system tray application with a unified architecture:

```bash
# Launch the wallet (after building with dev.sh rebuild)
thresh

# Or run in development mode
cd cip-30-wallet
npm run tauri dev
```

The app will:
- ✅ Start in the system tray (look for the icon in your system tray)
- ✅ Handle both manual UI access and browser extension communication
- ✅ Prevent multiple instances (single-instance architecture)
- ✅ Log all activity to `/tmp/thresh.log`

## Development

### Project Structure

```
tauri-wallet-extension/
├── chrome-extension/        # Browser extension
│   ├── manifest.json       # Extension manifest (V3)
│   ├── background.js       # Native messaging handler
│   ├── content.js          # Injects CIP-30 API
│   └── popup.html/js       # Extension UI
├── cip-30-wallet/          # Desktop wallet application
│   ├── src/               # React frontend
│   └── src-tauri/         # Rust backend
│       └── src/
│           ├── cip30/     # CIP-30 API implementation
│           ├── crypto/    # Encryption & key management
│           ├── storage/   # Wallet persistence
│           ├── utxorpc/   # Blockchain interaction
│           └── wallet/    # Core wallet logic
├── dolos/                  # Dolos configuration
│   ├── dolos.toml         # Node configuration
│   └── data/              # Blockchain data
└── dev.sh                  # Development helper script
```

### Development Commands

```bash
# Build debug version
./dev.sh build

# Build release version
./dev.sh build-release

# Rebuild and reinstall (recommended for testing)
./dev.sh rebuild YOUR_EXTENSION_ID

# Monitor logs (real-time)
./dev.sh logs

# View recent log entries
./logs.sh show

# Clear logs
./logs.sh clear

# Show development status
./dev.sh status

# Test native messaging
./dev.sh test-native
```

### Setting Up Development Environment

1. **Frontend Development** (React/TypeScript):
```bash
cd cip-30-wallet
npm run dev
```

2. **Backend Development** (Rust):
```bash
cd cip-30-wallet/src-tauri
cargo watch -x check -x test -x run
```

3. **Extension Development**:
- Make changes to files in `chrome-extension/`
- Refresh extension in `chrome://extensions/`

## CIP-30 Implementation Status

### ✅ Implemented Methods

- [x] `getNetworkId()` - Returns network ID (1=mainnet, 0=testnet)
- [x] `getUsedAddresses()` - Returns used wallet addresses
- [x] `getUnusedAddresses()` - Returns unused wallet addresses
- [x] `getChangeAddress()` - Returns change address
- [x] `getRewardAddresses()` - Returns stake reward addresses
- [x] `getExtensions()` - Returns supported extensions
- [x] `getUtxos()` - Fetches UTxOs via UTxO RPC
- [x] `getBalance()` - Calculates wallet balance

### ❌ Not Yet Implemented

- [ ] `signTx()` - Transaction signing (mock implementation only)
- [ ] `signData()` - Data signing (mock implementation only)
- [ ] `submitTx()` - Transaction submission (mock implementation only)

## Roadmap

### Current Development Focus

- [ ] Display balance and assets in desktop UI
- [ ] Create transactions from desktop UI
- [ ] Caching and indexing assets and transactions
- [ ] Integrate Tx3 protocols (auto-generation of forms from protocol definitions)

### Future Plans: FROST Threshold Signatures

One of the key motivations for creating Thresh is to eventually implement FROST (Flexible Round-Optimized Schnorr Threshold) signatures, which will enable:

**Distributed Key Generation & Management**
- Multiple parties can collectively control a wallet
- No single point of failure for key material
- Dynamic threshold configuration without address changes

**Key Benefits:**
- **dApp Compatibility**: Multisigs appear as regular wallets to dApps, not as scripts
- **Dynamic Configuration**: Modify thresholds and signers without changing the wallet address
- **Enhanced Security**: Distribute trust across multiple parties or devices
- **Seamless UX**: Users interact with dApps normally while benefiting from multisig security

## Usage

### For End Users

1. Launch the Thresh desktop application
2. Create or import a wallet using your mnemonic phrase
3. Connect to your preferred network (mainnet/testnet/devnet)
4. Click the browser extension icon and verify connection
5. Visit any CIP-30 compatible Cardano dApp
6. The dApp will detect "Thresh" as an available wallet

### For dApp Developers

Thresh exposes the standard CIP-30 API at `window.cardano.thresh`:

```javascript
// Check if Thresh is available
if (window.cardano && window.cardano.thresh) {
  const wallet = window.cardano.thresh;
  
  // Request wallet access
  const api = await wallet.enable();
  
  // Use wallet API
  const networkId = await api.getNetworkId();
  const balance = await api.getBalance();
  const utxos = await api.getUtxos();
  const addresses = await api.getUsedAddresses();
  
  // Coming soon: transaction operations
  // const signedTx = await api.signTx(tx);
  // const txHash = await api.submitTx(signedTx);
}
```

## Running Your Own Devnet

Thresh + Dolos makes it easy to run isolated development environments:

1. Configure Dolos for devnet in `dolos/dolos.toml`
2. Start Dolos: `dolos daemon`
3. Configure Thresh to connect to your local Dolos instance
4. Deploy and test your dApps in a controlled environment

## Architecture

### System Tray Application

Thresh uses a **unified system tray architecture**:

- **Single Process**: One binary handles both UI and browser extension communication
- **System Tray**: App runs in background, accessible via tray icon  
- **Single Instance**: Only one instance runs at a time (prevents multiple tray icons)
- **Always Available**: Responds to both manual launches and browser extension requests

### Communication Flow

1. **Browser Extension** → Native Messaging → **Thresh Binary**
2. **User Clicks Tray** → **Thresh UI** (same process)  
3. **Transaction Signing** → Shows password dialog in existing window

## Troubleshooting

### Extension Connection Issues

1. **Verify Native Messaging Host**:
```bash
# Check if manifest exists
ls ~/.config/google-chrome/NativeMessagingHosts/com.cardano.thresh.json

# Verify extension ID matches
cat ~/.config/google-chrome/NativeMessagingHosts/com.cardano.thresh.json
```

2. **Check Logs**:
```bash
# Monitor logs in real-time
./logs.sh

# View recent log entries
./logs.sh show

# Clear logs
./logs.sh clear

# View Chrome console for extension errors
# Go to chrome://extensions/ → Details → Background page
```

3. **Test Native Messaging**:
```bash
./dev.sh test-native
```

### Dolos Connection Issues

1. Ensure Dolos is running: `ps aux | grep dolos`
2. Check Dolos logs in the data directory
3. Verify network configuration in `dolos.toml`

## Contributing

We welcome contributions! Please see our contributing guidelines for:
- Code style and standards
- Testing requirements
- Pull request process
- Security considerations

## License

[License information here]

## Acknowledgments

Thresh is built on top of excellent open-source projects:
- [Dolos](https://github.com/txpipe/dolos) - Lightweight Cardano node
- [Pallas](https://github.com/txpipe/pallas) - Rust Cardano primitives
- [Tauri](https://tauri.app) - Secure desktop app framework
- [CIP-30](https://cips.cardano.org/cips/cip30/) - dApp connector standard