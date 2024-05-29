

# Thresh - A Desktop Cardano Wallet

## Introduction

**Thresh** is a desktop Cardano wallet that leverages FROST threshold signatures and distributed key generation. This innovative approach enables Cardano users to manage multisignature (multisig) wallets that can seamlessly interact with decentralized applications (dApps) without relying on Native Scripts. 

### Key Benefits

- **dApp Compatibility**: Multisigs can interact with any dApp, with addresses and signatures appearing as regular wallets, not scripts.
- **Dynamic Configuration**: Modify thresholds and signers without changing the public key, ensuring the wallet address remains the same.

## Features

- **Multisig Interaction with dApps**: Utilize multisig wallets to interact with any dApp.
- **Dynamic Threshold and Signer Management**: Modify thresholds and signers seamlessly.
- **Distributed Key Generation (DKG)**: Each user can generate their keys and synchronize with others to complete all FROST DKG rounds.
- **Multi-Account Management**: Manage multiple wallets, each user can be part of various multisigs.
- **User-Friendly Transaction Management**: Manage UTxOs to avoid reuse in pending transactions.

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/)
- [Rust](https://www.rust-lang.org/)
- [Tauri](https://tauri.app/)

### Installation

1. Clone the repository:
   ```sh
   git clone https://github.com/your-repo/thresh.git
   cd thresh
   ```

2. Install dependencies:
   ```sh
   npm install
   ```

3. Build the project:
   ```sh
   npm run build
   ```

4. Run the application:
   ```sh
   npm run tauri dev
   ```


## Contributing

We welcome contributions from the community! Here’s how you can get involved:

1. Fork the repository.
2. Create a new branch for your feature or bugfix.
3. Commit your changes.
4. Push the branch to your fork.
5. Create a pull request.

## Roadmap

- [x] Instantiate webview window with a URL entered by the user or whitelist some dApp domains.
- [ ] Implement multiple wallet management.
- [ ] Complete distributed key generation and synchronization.
- [ ] Write CIP30 functions for FROST signing and signature synchronization.
- [x] Inject CIP30 API into the webview window.
- [ ] Implement UTxO management to prevent reuse in pending transactions.
- [ ] Explore transaction chaining.
- [ ] Consider removing Blockfrost dependency by integrating with a local node (e.g., dolos).

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contact

For any questions or suggestions, please open an issue.
