#!/bin/bash

# Installation script for Chrome Native Messaging Host

set -e

echo "Installing Tauri Wallet Native Messaging Host..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get the directory of this script
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

# Check if Chrome extension ID is provided
if [ -z "$1" ]; then
    echo -e "${YELLOW}Usage: $0 <chrome-extension-id>${NC}"
    echo "You can find your extension ID in Chrome at chrome://extensions/"
    exit 1
fi

EXTENSION_ID=$1

# Build the Tauri app
echo "Building Tauri app..."
cd "$SCRIPT_DIR/cip-30-wallet/src-tauri"
cargo build --release -q

# Detect OS
OS=$(uname -s)
case "$OS" in
    Linux*)
        TARGET_DIR="$HOME/.config/google-chrome/NativeMessagingHosts"
        BINARY_PATH="$SCRIPT_DIR/cip-30-wallet/src-tauri/target/release/cip-30-wallet"
        ;;
    Darwin*)
        TARGET_DIR="$HOME/Library/Application Support/Google/Chrome/NativeMessagingHosts"
        BINARY_PATH="$SCRIPT_DIR/cip-30-wallet/src-tauri/target/release/cip-30-wallet"
        ;;
    MINGW*|CYGWIN*|MSYS*)
        echo -e "${RED}Windows installation requires manual steps. Please see README.md${NC}"
        exit 1
        ;;
    *)
        echo -e "${RED}Unsupported OS: $OS${NC}"
        exit 1
        ;;
esac

# Create native messaging hosts directory if it doesn't exist
mkdir -p "$TARGET_DIR"

# Create a user-local directory for the binary
USER_BIN_DIR="$HOME/.local/bin"
mkdir -p "$USER_BIN_DIR"

# Copy binary to user-local location (no sudo needed)
INSTALL_PATH="$USER_BIN_DIR/cip-30-wallet"
echo "Installing binary to $INSTALL_PATH..."
cp "$BINARY_PATH" "$INSTALL_PATH"
chmod +x "$INSTALL_PATH"

# Create wrapper script that adds --native-messaging flag and logging
WRAPPER_PATH="$USER_BIN_DIR/cip-30-wallet-native"
cat > "$WRAPPER_PATH" << EOF
#!/bin/bash
LOG_FILE="/tmp/tauri-wallet-native-messaging.log"
echo "\$(date): Native messaging host started" >> "\$LOG_FILE"
echo "\$(date): Arguments: \$@" >> "\$LOG_FILE"
exec "$INSTALL_PATH" --native-messaging 2>> "\$LOG_FILE"
EOF
chmod +x "$WRAPPER_PATH"

# Create native messaging manifest with correct extension ID
MANIFEST_PATH="$TARGET_DIR/com.cardano.tauri_wallet.json"
cat > "$MANIFEST_PATH" << EOF
{
  "name": "com.cardano.tauri_wallet",
  "description": "Tauri-based Cardano Wallet Native Messaging Host",
  "path": "$WRAPPER_PATH",
  "type": "stdio",
  "allowed_origins": [
    "chrome-extension://$EXTENSION_ID/"
  ]
}
EOF

echo -e "${GREEN}✓ Native messaging host installed successfully!${NC}"
echo
echo "Installation complete. The native messaging host has been registered."
echo "Manifest location: $MANIFEST_PATH"
echo "Binary location: $INSTALL_PATH"
echo
echo -e "${YELLOW}Next steps:${NC}"
echo "1. Load the Chrome extension from: $SCRIPT_DIR/chrome-extension"
echo "2. The extension should now be able to communicate with the Tauri app"
echo
echo "To test the connection:"
echo "1. Open the extension popup"
echo "2. Click 'Connect to Tauri App'"
echo "3. Check the status indicator"