# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

### Development
- `npm run dev` - Start the frontend development server (Vite)
- `npm run tauri dev` - Start the Tauri development server (runs both frontend and backend)
- `npm run build` - Build the frontend for production
- `npm run tauri build` - Build the complete application for distribution

### Testing
- `cargo test` - Run Rust tests in src-tauri/
- `cargo test -- --nocapture` - Run Rust tests with output

### Rust Backend
- `cd src-tauri && cargo check` - Check Rust code for errors without building
- `cd src-tauri && cargo fmt` - Format Rust code
- `cd src-tauri && cargo clippy` - Run Rust linter

## Architecture

This is a Tauri-based Cardano wallet implementing the CIP-30 standard. The application has two main components:

### Frontend (React/TypeScript)
- Entry point: `src/main.tsx`
- Main components flow: `WelcomeScreen` → `MnemonicGeneration` → `MnemonicConfirmation` → `WalletView`
- UI components use Radix UI primitives with Tailwind CSS
- State management through React hooks and Tauri commands

### Backend (Rust)
Located in `src-tauri/src/`:

1. **CIP-30 API Implementation** (`cip30/`)
   - Implements standard wallet methods: `getNetworkId`, `getUtxos`, `getBalance`, `getUsedAddresses`, `getChangeAddress`, `getRewardAddress`, `submitTx`
   - Entry point: `cip30/mod.rs`

2. **Wallet Core** (`wallet/`)
   - `manager.rs` - Main wallet operations and state management
   - `hd.rs` - HD wallet implementation using BIP-39/BIP-32
   - `account.rs` - Account management (multiple accounts per wallet)
   - `address.rs` - Address generation and management
   - `utxo.rs` - UTXO tracking and management

3. **Native Messaging** (`native_messaging.rs`)
   - Allows the wallet to act as a browser extension backend
   - Can run in headless mode for browser integration

4. **UTXO RPC Client** (`utxorpc/`)
   - Connects to Cardano blockchain via UTXO RPC protocol
   - Configuration in `config.toml`

### Key Design Patterns

1. **Tauri Commands**: All frontend-backend communication happens through Tauri commands defined in `main.rs`
2. **State Management**: Uses Tauri's managed state for wallet instance
3. **Error Handling**: Custom error types with proper serialization for frontend
4. **Security**: Wallets are password-protected with encrypted storage

### Important Files
- `src-tauri/tauri.conf.json` - Tauri configuration
- `config.toml.example` - Example configuration for UTXO RPC endpoints
- `src-tauri/src/lib.rs` - Main Tauri setup and command registration

### Native Messaging Mode
The wallet can run as a native messaging host:
```bash
cargo run --bin cip-30-wallet -- --native-messaging
```

This mode is used for browser extension integration and communicates via stdin/stdout.