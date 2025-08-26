#!/bin/bash

# Tauri Wallet Development Helper Script

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
EXTENSION_DIR="$SCRIPT_DIR/chrome-extension"
TAURI_DIR="$SCRIPT_DIR/cip-30-wallet/src-tauri"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_usage() {
    echo "Tauri Wallet Development Helper"
    echo ""
    echo "Usage: ./dev.sh [command]"
    echo ""
    echo "Commands:"
    echo "  build              Build the Tauri app (debug mode)"
    echo "  build-release      Build the Tauri app (release mode)"
    echo "  rebuild <ext-id>   Build Tauri app and reinstall native messaging host"
    echo "  install <ext-id>   Install native messaging host"
    echo "  logs               Follow logs in real-time"
    echo "  logs-show          Show recent log entries"
    echo "  logs-clear         Clear logs"
    echo "  test-native        Test native messaging manually"
    echo "  extension-info     Show extension loading instructions"
    echo "  status             Show current status"
    echo "  help               Show this help"
}

build_tauri() {
    local mode="$1"
    echo -e "${BLUE}Building Tauri app in $mode mode...${NC}"
    cd "$TAURI_DIR"
    if [ "$mode" = "release" ]; then
        cargo build --release
    else
        cargo build
    fi
    cd "$SCRIPT_DIR"
}

show_status() {
    echo -e "${BLUE}=== Tauri Wallet Development Status ===${NC}"
    echo ""
    
    # Check if extension files exist
    if [ -f "$EXTENSION_DIR/manifest.json" ]; then
        echo -e "${GREEN}✓${NC} Chrome extension files found"
    else
        echo -e "${RED}✗${NC} Chrome extension files missing"
    fi
    
    # Check if Tauri binary exists
    if [ -f "$TAURI_DIR/target/release/cip-30-wallet" ]; then
        echo -e "${GREEN}✓${NC} Tauri app (release) built"
    elif [ -f "$TAURI_DIR/target/debug/cip-30-wallet" ]; then
        echo -e "${YELLOW}△${NC} Tauri app (debug) built"
    else
        echo -e "${RED}✗${NC} Tauri app not built"
    fi
    
    # Check native messaging manifest
    if [ -f "$HOME/.config/google-chrome/NativeMessagingHosts/com.cardano.tauri_wallet.json" ]; then
        echo -e "${GREEN}✓${NC} Native messaging host installed"
        echo "    Extension ID: $(grep -o 'chrome-extension://[^/]*' ~/.config/google-chrome/NativeMessagingHosts/com.cardano.tauri_wallet.json | sed 's/chrome-extension:\/\///')"
    else
        echo -e "${RED}✗${NC} Native messaging host not installed"
    fi
    
    # Check if log file exists
    if [ -f "/tmp/tauri-wallet-native-messaging.log" ]; then
        echo -e "${GREEN}✓${NC} Log file exists ($(wc -l < /tmp/tauri-wallet-native-messaging.log) lines)"
    else
        echo -e "${YELLOW}△${NC} No log file (extension not used yet)"
    fi
    
    echo ""
    echo -e "${BLUE}Next steps:${NC}"
    echo "1. Load extension in Chrome from: $EXTENSION_DIR"
    echo "2. Get extension ID and run: ./dev.sh install <extension-id>"
    echo "3. Test connection with extension popup"
    echo "4. Monitor logs with: ./dev.sh logs"
}

case "${1:-help}" in
    "build")
        build_tauri "debug"
        ;;
        
    "build-release")
        build_tauri "release"
        ;;
        
    "rebuild")
        if [ -z "$2" ]; then
            echo -e "${RED}Error: Extension ID required${NC}"
            echo "Usage: ./dev.sh rebuild <extension-id>"
            echo "Find your extension ID in chrome://extensions/"
            exit 1
        fi
        echo -e "${BLUE}=== Rebuild and Reinstall ===${NC}"
        echo "Building Tauri app and reinstalling native messaging host..."
        echo ""
        
        # Build the app
        build_tauri "release"
        
        if [ $? -eq 0 ]; then
            echo ""
            echo -e "${BLUE}Build successful! Installing native messaging host...${NC}"
            ./install-native-host.sh "$2"
            
            if [ $? -eq 0 ]; then
                echo ""
                echo -e "${GREEN}✓ Rebuild and reinstall completed successfully!${NC}"
                echo -e "${YELLOW}Next steps:${NC}"
                echo "1. If you made Chrome extension changes, reload the extension in chrome://extensions/"
                echo "2. Test your changes in the browser"
                echo "3. Monitor logs with: ./dev.sh logs"
            else
                echo -e "${RED}✗ Installation failed${NC}"
                exit 1
            fi
        else
            echo -e "${RED}✗ Build failed${NC}"
            exit 1
        fi
        ;;
        
    "install")
        if [ -z "$2" ]; then
            echo -e "${RED}Error: Extension ID required${NC}"
            echo "Usage: ./dev.sh install <extension-id>"
            echo "Find your extension ID in chrome://extensions/"
            exit 1
        fi
        ./install-native-host.sh "$2"
        ;;
        
    "logs")
        ./logs.sh follow
        ;;
        
    "logs-show")
        ./logs.sh show
        ;;
        
    "logs-clear")
        ./logs.sh clear
        ;;
        
    "test-native")
        if [ -f "$HOME/.local/bin/cip-30-wallet-native" ]; then
            echo -e "${BLUE}Testing native messaging host...${NC}"
            echo '{"type":"ping","requestId":123}' | python3 -c "
import json
import struct
import subprocess
import sys

message = sys.stdin.read().strip()
encoded = message.encode('utf-8')
length = struct.pack('<I', len(encoded))

proc = subprocess.Popen(['$HOME/.local/bin/cip-30-wallet-native'],
                       stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
proc.stdin.write(length + encoded)
proc.stdin.close()

# Try to read response
try:
    resp_len = proc.stdout.read(4)
    if len(resp_len) == 4:
        length = struct.unpack('<I', resp_len)[0]
        response = proc.stdout.read(length)
        print('Response:', response.decode('utf-8'))
    else:
        print('No response received')
except:
    print('Error reading response')

proc.wait()
print('Exit code:', proc.returncode)
"
        else
            echo -e "${RED}Native messaging host not installed${NC}"
        fi
        ;;
        
    "extension-info")
        echo -e "${BLUE}Chrome Extension Loading Instructions:${NC}"
        echo ""
        echo "1. Open Chrome and go to: chrome://extensions/"
        echo "2. Enable 'Developer mode' (toggle in top-right)"
        echo "3. Click 'Load unpacked'"
        echo "4. Select this directory: $EXTENSION_DIR"
        echo "5. Note the Extension ID (e.g., abcdef123456...)"
        echo "6. Run: ./dev.sh install <extension-id>"
        echo ""
        echo -e "${YELLOW}Tip:${NC} After making changes to the extension, click the refresh icon in chrome://extensions/"
        ;;
        
    "status")
        show_status
        ;;
        
    "help"|"-h"|"--help")
        print_usage
        ;;
        
    *)
        echo -e "${RED}Unknown command: $1${NC}"
        print_usage
        exit 1
        ;;
esac