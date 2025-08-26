#!/bin/bash
# Debug wrapper for native messaging host

LOG_FILE="/tmp/tauri-wallet-native-messaging.log"

echo "$(date): Native messaging host started" >> "$LOG_FILE"
echo "$(date): Arguments: $@" >> "$LOG_FILE"
echo "$(date): Environment:" >> "$LOG_FILE"
env >> "$LOG_FILE"
echo "$(date): Starting Tauri app..." >> "$LOG_FILE"

# Run the actual binary and log stderr
/usr/local/bin/cip-30-wallet --native-messaging 2>> "$LOG_FILE"

echo "$(date): Native messaging host exited with code $?" >> "$LOG_FILE"