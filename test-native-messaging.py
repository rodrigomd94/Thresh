#!/usr/bin/env python3
import json
import struct
import subprocess
import sys

def send_native_message(message):
    """Send a message using native messaging protocol"""
    # Encode message as JSON
    encoded_message = json.dumps(message).encode('utf-8')
    
    # Create length header (4 bytes, little-endian)
    length = struct.pack('<I', len(encoded_message))
    
    # Run the native messaging host
    proc = subprocess.Popen(['/usr/local/bin/cip-30-wallet-native'], 
                           stdin=subprocess.PIPE, 
                           stdout=subprocess.PIPE,
                           stderr=subprocess.PIPE)
    
    # Send length + message
    proc.stdin.write(length)
    proc.stdin.write(encoded_message)
    proc.stdin.flush()
    
    # Read response length
    response_length_bytes = proc.stdout.read(4)
    if len(response_length_bytes) < 4:
        print("Failed to read response length")
        return
    
    response_length = struct.unpack('<I', response_length_bytes)[0]
    print(f"Response length: {response_length}")
    
    # Read response
    response_data = proc.stdout.read(response_length)
    response = json.loads(response_data)
    
    print(f"Response: {json.dumps(response, indent=2)}")
    
    # Check stderr
    stderr = proc.stderr.read().decode('utf-8')
    if stderr:
        print(f"Stderr: {stderr}")

# Test ping
print("Testing ping...")
send_native_message({"type": "ping", "requestId": 123})